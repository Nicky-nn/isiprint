import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { verifySession, logout, isTauri } from "./api";
import { changeLanguage } from "./i18n";
import { LoginScreen } from "./components/LoginScreen";
import { AccountTab } from "./components/AccountTab";
import { PrintersTab } from "./components/PrintersTab";
import { LogsTab } from "./components/LogsTab";
import { SimpleIcon } from "./components/LordIcon";
import { StaticLogo, AnimatedLogo } from "./components/AnimatedLogo";
import type { AuthState } from "./types";
import "./App.css";

// Import i18n
import "./i18n";

type TabType = "account" | "printers" | "logs";

function App() {
    const { t, i18n } = useTranslation();
    const [activeTab, setActiveTab] = useState<TabType>("account");
    const [authState, setAuthState] = useState<AuthState>({
        token: null,
        refresh_token: null,
        email: null,
        is_logged_in: false,
    });
    const [isVerifying, setIsVerifying] = useState(true);

    // Verify saved session on startup
    useEffect(() => {
        const checkSession = async () => {
            // Skip if not in Tauri environment
            if (!isTauri()) {
                setIsVerifying(false);
                return;
            }

            try {
                setIsVerifying(true);
                const result = await verifySession();
                if (result.success && result.data) {
                    setAuthState(result.data);
                } else {
                    setAuthState({
                        token: null,
                        refresh_token: null,
                        email: null,
                        is_logged_in: false,
                    });
                }
            } catch (err) {
                // Still set not logged in on error
                setAuthState({
                    token: null,
                    refresh_token: null,
                    email: null,
                    is_logged_in: false,
                });
            } finally {
                setIsVerifying(false);
            }
        };
        checkSession();
    }, []);

    // Listen for tray navigation events (only in Tauri)
    useEffect(() => {
        if (!isTauri()) return;

        let unlistenFn: (() => void) | null = null;

        const setupListener = async () => {
            try {
                const { listen } = await import("@tauri-apps/api/event");
                unlistenFn = await listen<string>("navigate", (event) => {
                    const tab = event.payload as TabType;
                    if (["account", "printers", "logs"].includes(tab)) {
                        setActiveTab(tab);
                    }
                });
            } catch (err) {}
        };

        setupListener();

        return () => {
            if (unlistenFn) unlistenFn();
        };
    }, []);

    const handleLogout = async () => {
        try {
            await logout();
            setAuthState({
                token: null,
                refresh_token: null,
                email: null,
                is_logged_in: false,
            });
        } catch (err) {}
    };

    const handleLanguageChange = (lang: string) => {
        changeLanguage(lang);
    };

    const languages = [
        { code: "es", label: "Español" },
        { code: "en", label: "English" },
        { code: "fr", label: "Français" },
    ];

    const tabs = [
        { id: "account" as TabType, icon: "user", label: t("nav.account") },
        {
            id: "printers" as TabType,
            icon: "printer",
            label: t("nav.printers"),
        },
        { id: "logs" as TabType, icon: "logs", label: t("nav.logs") },
    ];

    // Show loading screen while verifying session
    if (isVerifying) {
        return (
            <div
                style={{
                    minHeight: "100vh",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    backgroundColor: "#0a0a0a",
                    color: "#ffffff",
                    gap: "32px",
                }}
            >
                <AnimatedLogo
                    size={100}
                    color="#88FCA4"
                    isAnimated={true}
                    showText={true}
                    text="Iniciando..."
                    subText="Verificando sesión"
                />
            </div>
        );
    }

    // Show login screen if not authenticated
    if (!authState.is_logged_in) {
        return <LoginScreen onLoginSuccess={setAuthState} />;
    }

    // Main app (authenticated)
    return (
        <div
            className="app-container"
            style={{
                display: "flex",
                height: "100vh",
                width: "100vw",
                backgroundColor: "#0a0a0a",
                color: "#ffffff",
                position: "fixed",
                top: 0,
                left: 0,
            }}
        >
            {/* Sidebar */}
            <motion.aside
                className="sidebar"
                style={{
                    backgroundColor: "#111111",
                    width: "260px",
                    height: "100vh",
                    flexShrink: 0,
                    display: "flex",
                    flexDirection: "column",
                    padding: "20px 16px",
                    borderRight: "1px solid rgba(255,255,255,0.08)",
                }}
                initial={{ x: -100, opacity: 0 }}
                animate={{ x: 0, opacity: 1 }}
                transition={{ duration: 0.4 }}
            >
                {/* Logo */}
                <div
                    className="sidebar-header"
                    style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "12px",
                        marginBottom: "32px",
                    }}
                >
                    <div
                        className="sidebar-logo"
                        style={{
                            width: "44px",
                            height: "44px",
                            background: "#0a0a0a",
                            border: "2px solid #88FCA4",
                            borderRadius: "12px",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                        }}
                    >
                        <StaticLogo size={28} color="#88FCA4" />
                    </div>
                    <span
                        className="sidebar-title"
                        style={{
                            fontSize: "20px",
                            fontWeight: 700,
                            color: "#ffffff",
                        }}
                    >
                        ISIPRINT
                    </span>
                </div>

                {/* Navigation */}
                <nav
                    className="sidebar-nav"
                    style={{
                        display: "flex",
                        flexDirection: "column",
                        gap: "6px",
                        flex: 1,
                    }}
                >
                    {tabs.map((tab) => (
                        <motion.button
                            key={tab.id}
                            className={`nav-item ${
                                activeTab === tab.id ? "active" : ""
                            }`}
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "12px",
                                padding: "12px 16px",
                                border: "none",
                                background:
                                    activeTab === tab.id
                                        ? "rgba(136, 252, 164, 0.1)"
                                        : "transparent",
                                borderRadius: "12px",
                                cursor: "pointer",
                                color:
                                    activeTab === tab.id
                                        ? "#88FCA4"
                                        : "rgba(255,255,255,0.6)",
                                fontSize: "14px",
                                textAlign: "left",
                            }}
                            onClick={() => setActiveTab(tab.id)}
                            whileHover={{ x: 4 }}
                            whileTap={{ scale: 0.98 }}
                        >
                            <SimpleIcon
                                icon={tab.icon as any}
                                size={20}
                                color={
                                    activeTab === tab.id ? "#88FCA4" : "#666666"
                                }
                            />
                            <span>{tab.label}</span>
                            {activeTab === tab.id && (
                                <motion.div
                                    className="nav-indicator"
                                    layoutId="nav-indicator"
                                    transition={{
                                        type: "spring",
                                        stiffness: 500,
                                        damping: 30,
                                    }}
                                />
                            )}
                        </motion.button>
                    ))}
                </nav>

                {/* Sidebar footer */}
                <div className="sidebar-footer">
                    {/* Language selector */}
                    <select
                        value={i18n.language}
                        onChange={(e) => handleLanguageChange(e.target.value)}
                        style={{
                            height: "36px",
                            borderRadius: "10px",
                            padding: "0 12px",
                            border: "1px solid rgba(255,255,255,0.15)",
                            background: "rgba(255,255,255,0.04)",
                            color: "#ffffff",
                            cursor: "pointer",
                            fontSize: "13px",
                            outline: "none",
                            appearance: "none",
                            WebkitAppearance: "none",
                        }}
                    >
                        {languages.map((lang) => (
                            <option
                                key={lang.code}
                                value={lang.code}
                                style={{
                                    background: "#0a0a0a",
                                    color: "#ffffff",
                                }}
                            >
                                {lang.label}
                            </option>
                        ))}
                    </select>

                    {/* User info */}
                    <div
                        className="user-info"
                        style={{
                            display: "flex",
                            alignItems: "center",
                            gap: "12px",
                            padding: "12px",
                            background: "#1a1a1a",
                            borderRadius: "12px",
                            marginBottom: "12px",
                        }}
                    >
                        <div
                            className="user-avatar"
                            style={{
                                width: "40px",
                                height: "40px",
                                background: "rgba(136, 252, 164, 0.1)",
                                borderRadius: "10px",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                            }}
                        >
                            <SimpleIcon icon="user" size={20} color="#88FCA4" />
                        </div>
                        <div
                            className="user-details"
                            style={{ flex: 1, minWidth: 0 }}
                        >
                            <span
                                className="user-email"
                                style={{
                                    display: "block",
                                    fontSize: "13px",
                                    fontWeight: 500,
                                    color: "#ffffff",
                                    whiteSpace: "nowrap",
                                    overflow: "hidden",
                                    textOverflow: "ellipsis",
                                }}
                            >
                                {authState.email}
                            </span>
                            <span
                                className="user-status"
                                style={{
                                    fontSize: "11px",
                                    color: "#88FCA4",
                                }}
                            >
                                {t("status.connected")}
                            </span>
                        </div>
                    </div>

                    {/* Logout */}
                    <motion.button
                        className="logout-btn"
                        style={{
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            gap: "8px",
                            padding: "12px",
                            width: "100%",
                            background: "rgba(248, 253, 103, 0.1)",
                            border: "1px solid rgba(248, 253, 103, 0.2)",
                            borderRadius: "12px",
                            color: "#F8FD67",
                            cursor: "pointer",
                            fontSize: "14px",
                            fontWeight: 500,
                        }}
                        onClick={handleLogout}
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                    >
                        <SimpleIcon icon="logout" size={18} color="#F8FD67" />
                        <span>{t("auth.logout")}</span>
                    </motion.button>
                </div>
            </motion.aside>

            {/* Main content */}
            <main
                className="main-content"
                style={{
                    flex: 1,
                    backgroundColor: "#0a0a0a",
                    color: "#ffffff",
                    padding: "32px",
                    overflowY: "auto",
                }}
            >
                <AnimatePresence mode="wait">
                    <motion.div
                        key={activeTab}
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -20 }}
                        transition={{ duration: 0.3 }}
                        className="tab-content"
                        style={{
                            color: "#ffffff",
                            maxWidth: "900px",
                            margin: "0 auto",
                        }}
                    >
                        {activeTab === "account" && (
                            <AccountTab authState={authState} />
                        )}
                        {activeTab === "printers" && <PrintersTab />}
                        {activeTab === "logs" && <LogsTab />}
                    </motion.div>
                </AnimatePresence>
            </main>
        </div>
    );
}

export default App;
