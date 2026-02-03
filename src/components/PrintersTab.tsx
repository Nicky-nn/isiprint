import { useMemo, useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import {
    getPrinters,
    printTestPage,
    sendCutCommand,
    scanNetworkPrinters,
    addNetworkPrinter,
    getLocalIp,
} from "../api";
import { SimpleIcon } from "./LordIcon";
import { AnimatedLogo } from "./AnimatedLogo";
import type { PrintSettings, NetworkPrinter } from "../types";
import "./PrintersTab.css";

export function PrintersTab() {
    const { t, i18n } = useTranslation();
    const [printers, setPrinters] = useState<string[]>([]);
    const [selectedPrinter, setSelectedPrinter] = useState<string>("");
    const [isLoading, setIsLoading] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [printerSettingsByName, setPrinterSettingsByName] = useState<
        Record<string, PrintSettings>
    >({});
    const [message, setMessage] = useState<{
        type: "success" | "error";
        text: string;
    } | null>(null);

    // Network discovery states
    const [isScanning, setIsScanning] = useState(false);
    const [networkPrinters, setNetworkPrinters] = useState<NetworkPrinter[]>([]);
    const [localIp, setLocalIp] = useState<string>("");
    const [showNetworkDiscovery, setShowNetworkDiscovery] = useState(false);

    const defaultSettings: PrintSettings = useMemo(
        () => ({ preset: "thermal", width_mm: 80, height_mm: 200 }),
        []
    );

    const currentSettings: PrintSettings = useMemo(() => {
        if (!selectedPrinter) return defaultSettings;
        return printerSettingsByName[selectedPrinter] ?? defaultSettings;
    }, [defaultSettings, printerSettingsByName, selectedPrinter]);

    const saveSettings = (printerName: string, settings: PrintSettings) => {
        const next = { ...printerSettingsByName, [printerName]: settings };
        setPrinterSettingsByName(next);
        try {
            localStorage.setItem(
                "isiprint.printerSettings",
                JSON.stringify(next)
            );
        } catch {
            // ignore
        }
    };

    useEffect(() => {
        try {
            const raw = localStorage.getItem("isiprint.printerSettings");
            if (raw) {
                const parsed = JSON.parse(raw) as Record<string, PrintSettings>;
                if (parsed && typeof parsed === "object")
                    setPrinterSettingsByName(parsed);
            }
        } catch {
            // ignore
        }
    }, []);

    useEffect(() => {
        loadPrinters();
    }, []);

    const loadPrinters = async () => {
        try {
            setIsLoading(true);
            const response = await getPrinters();
            if (response.success && response.data) {
                setPrinters(response.data);
                if (response.data.length > 0 && !selectedPrinter) {
                    setSelectedPrinter(response.data[0]);
                }
            }
        } catch (err) {
        } finally {
            setIsLoading(false);
        }
    };

    const handleRefresh = async () => {
        setIsRefreshing(true);
        await loadPrinters();
        setIsRefreshing(false);
    };

    const handleTestPrint = async () => {
        if (!selectedPrinter) {
            setMessage({ type: "error", text: t("printers.selectFirst") });
            return;
        }

        try {
            setIsLoading(true);
            setMessage({ type: "success", text: t("printers.printing") });

            const response = await printTestPage(
                selectedPrinter,
                currentSettings,
                i18n.language
            );

            if (response.success) {
                setMessage({
                    type: "success",
                    text: response.data || t("printers.printSuccess"),
                });
            } else {
                setMessage({
                    type: "error",
                    text: response.error || t("printers.printError"),
                });
            }
        } catch (err) {
            setMessage({ type: "error", text: t("printers.printError") });
        } finally {
            setIsLoading(false);
        }
    };

    const handleCut = async () => {
        if (!selectedPrinter) {
            setMessage({ type: "error", text: t("printers.selectFirst") });
            return;
        }

        try {
            const response = await sendCutCommand(selectedPrinter);
            if (response.success) {
                setMessage({
                    type: "success",
                    text: response.data || "Cut command sent",
                });
            } else {
                setMessage({
                    type: "error",
                    text: response.error || "Error sending cut",
                });
            }
        } catch (err) {
            setMessage({ type: "error", text: "Error sending cut command" });
        }
    };

    // Network discovery functions
    const handleScanNetwork = async () => {
        setIsScanning(true);
        setMessage({ type: "success", text: t("printers.scanning") || "Escaneando red..." });

        try {
            // Get local IP first
            const ipResponse = await getLocalIp();
            if (ipResponse.success && ipResponse.data) {
                setLocalIp(ipResponse.data);
            }

            // Scan network for printers
            const response = await scanNetworkPrinters();

            if (response.success && response.data) {
                setNetworkPrinters(response.data);
                setShowNetworkDiscovery(true);
                setMessage({
                    type: "success",
                    text: `${t("printers.found") || "Encontradas"} ${response.data.length} ${t("printers.networkPrinters") || "impresoras en red"}`,
                });
            } else {
                setMessage({
                    type: "error",
                    text: response.error || t("printers.scanError") || "Error al escanear la red",
                });
            }
        } catch (err) {
            setMessage({
                type: "error",
                text: t("printers.scanError") || "Error al escanear la red",
            });
        } finally {
            setIsScanning(false);
        }
    };

    const handleAddNetworkPrinter = async (printer: NetworkPrinter) => {
        try {
            const response = await addNetworkPrinter(printer);
            if (response.success) {
                setMessage({
                    type: "success",
                    text: t("printers.printerAdded") || `Impresora ${printer.name} agregada`,
                });
                // Refresh printers list
                await loadPrinters();
            } else {
                setMessage({
                    type: "error",
                    text: response.error || "Error al agregar impresora",
                });
            }
        } catch (err) {
            setMessage({
                type: "error",
                text: "Error al agregar impresora de red",
            });
        }
    };

    useEffect(() => {

        if (message) {
            const timer = setTimeout(() => setMessage(null), 3000);
            return () => clearTimeout(timer);
        }
    }, [message]);

    const containerVariants = {
        hidden: { opacity: 0 },
        visible: {
            opacity: 1,
            transition: { staggerChildren: 0.1 },
        },
    };

    const itemVariants = {
        hidden: { opacity: 0, y: 20 },
        visible: { opacity: 1, y: 0 },
    };

    return (
        <motion.div
            className="printers-tab"
            style={{
                backgroundColor: "#0a0a0a",
                color: "#ffffff",
                minHeight: "100%",
            }}
            variants={containerVariants}
            initial="hidden"
            animate="visible"
        >
            {/* Header */}
            <motion.div className="page-header" variants={itemVariants}>
                <h1>{t("printers.title")}</h1>
                <p>{t("printers.available")}</p>
            </motion.div>

            {/* Message Toast */}
            <AnimatePresence>
                {message && (
                    <motion.div
                        className={`toast toast-${message.type}`}
                        initial={{ opacity: 0, y: -20 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -20 }}
                    >
                        <SimpleIcon
                            icon={
                                message.type === "success" ? "check" : "error"
                            }
                            size={18}
                            color={
                                message.type === "success"
                                    ? "#88FCA4"
                                    : "#F8FD67"
                            }
                        />
                        <span>{message.text}</span>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Printer Selection Card */}
            <motion.div className="card" variants={itemVariants}>
                <div className="card-header">
                    <h3 className="card-title">{t("printers.select")}</h3>
                    <motion.button
                        className="btn btn-secondary"
                        onClick={handleRefresh}
                        disabled={isRefreshing}
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                    >
                        <SimpleIcon
                            icon="refresh"
                            size={18}
                            color="#94a3b8"
                            className={isRefreshing ? "animate-spin" : ""}
                        />
                        {t("printers.refresh")}
                    </motion.button>
                </div>

                {isLoading ? (
                    <div
                        className="loading-state"
                        style={{
                            display: "flex",
                            flexDirection: "column",
                            alignItems: "center",
                            justifyContent: "center",
                            gap: "16px",
                            padding: "60px",
                        }}
                    >
                        <AnimatedLogo
                            size={80}
                            color="#88FCA4"
                            isAnimated={true}
                        />
                        <span style={{ color: "rgba(255,255,255,0.7)" }}>
                            {t("common.loading")}
                        </span>
                    </div>
                ) : printers.length === 0 ? (
                    <div className="empty-state">
                        <div className="empty-state-icon">
                            <SimpleIcon
                                icon="printer"
                                size={32}
                                color="#94a3b8"
                            />
                        </div>
                        <p>{t("printers.noPrinters")}</p>
                    </div>
                ) : (
                    <div className="printer-list">
                        {printers.map((printer, index) => (
                            <motion.div
                                key={printer}
                                className={`printer-item ${selectedPrinter === printer
                                    ? "selected"
                                    : ""
                                    }`}
                                onClick={() => setSelectedPrinter(printer)}
                                initial={{ opacity: 0, x: -20 }}
                                animate={{ opacity: 1, x: 0 }}
                                transition={{ delay: index * 0.05 }}
                                whileHover={{ scale: 1.01 }}
                            >
                                <div className="printer-icon">
                                    <SimpleIcon
                                        icon="printer"
                                        size={24}
                                        color={
                                            selectedPrinter === printer
                                                ? "#88FCA4"
                                                : "#94a3b8"
                                        }
                                    />
                                </div>
                                <div className="printer-info">
                                    <span className="printer-name">
                                        {printer}
                                    </span>
                                    {index === 0 && (
                                        <span className="printer-badge">
                                            {t("printers.default")}
                                        </span>
                                    )}
                                </div>
                                {selectedPrinter === printer && (
                                    <motion.div
                                        className="selected-indicator"
                                        layoutId="printer-selected"
                                        transition={{
                                            type: "spring",
                                            stiffness: 500,
                                            damping: 30,
                                        }}
                                    >
                                        <SimpleIcon
                                            icon="check"
                                            size={18}
                                            color="#88FCA4"
                                        />
                                    </motion.div>
                                )}
                            </motion.div>
                        ))}
                    </div>
                )}

                {/* Paper size settings per printer */}
                {!!selectedPrinter && (
                    <div
                        style={{
                            marginTop: "16px",
                            paddingTop: "16px",
                            borderTop: "1px solid rgba(255,255,255,0.08)",
                            display: "grid",
                            gridTemplateColumns: "1fr 1fr",
                            gap: "12px",
                        }}
                    >
                        <div
                            style={{
                                gridColumn: "span 2",
                                color: "rgba(255,255,255,0.7)",
                                fontSize: "12px",
                            }}
                        >
                            {t("printers.paperSize")}
                        </div>

                        <select
                            value={currentSettings.preset}
                            onChange={(e) => {
                                const preset = e.target
                                    .value as PrintSettings["preset"];
                                if (preset === "carta") {
                                    saveSettings(selectedPrinter, {
                                        preset: "carta",
                                    });
                                } else if (preset === "oficio") {
                                    saveSettings(selectedPrinter, {
                                        preset: "oficio",
                                    });
                                } else if (preset === "custom") {
                                    saveSettings(selectedPrinter, {
                                        preset: "custom",
                                        width_mm:
                                            currentSettings.width_mm ?? 80,
                                        height_mm:
                                            currentSettings.height_mm ?? 200,
                                    });
                                } else {
                                    saveSettings(selectedPrinter, {
                                        preset: "thermal",
                                        width_mm: 80,
                                        height_mm: 200,
                                    });
                                }
                            }}
                            style={{
                                height: "42px",
                                borderRadius: "12px",
                                padding: "0 40px 0 14px",
                                border: "1px solid rgba(255,255,255,0.18)",
                                background: `
            linear-gradient(180deg, #161616, #0b0b0b)
        `,
                                color: "#f5f5f5",
                                outline: "none",
                                appearance: "none",
                                WebkitAppearance: "none",
                                MozAppearance: "none",
                                cursor: "pointer",
                                boxShadow:
                                    "0 0 0 1px rgba(255,255,255,0.02), 0 4px 10px rgba(0,0,0,0.6)",
                                transition:
                                    "border 0.2s ease, box-shadow 0.2s ease",
                                backgroundImage: `
            url("data:image/svg+xml,%3Csvg width='16' height='16' viewBox='0 0 24 24' fill='white' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M7 10l5 5 5-5'/%3E%3C/svg%3E")
        `,
                                backgroundRepeat: "no-repeat",
                                backgroundPosition: "right 14px center",
                                backgroundSize: "16px",
                            }}
                            onFocus={(e) => {
                                e.currentTarget.style.border =
                                    "1px solid #3b82f6";
                                e.currentTarget.style.boxShadow =
                                    "0 0 0 2px rgba(59,130,246,0.35)";
                            }}
                            onBlur={(e) => {
                                e.currentTarget.style.border =
                                    "1px solid rgba(255,255,255,0.18)";
                                e.currentTarget.style.boxShadow =
                                    "0 0 0 1px rgba(255,255,255,0.02), 0 4px 10px rgba(0,0,0,0.6)";
                            }}
                        >
                            <option value="thermal">
                                {t("printers.paperThermal")}
                            </option>
                            <option value="carta">
                                {t("printers.paperCarta")}
                            </option>
                            <option value="oficio">
                                {t("printers.paperOficio")}
                            </option>
                            <option value="custom">
                                {t("printers.paperCustom")}
                            </option>
                        </select>

                        <div
                            style={{
                                display: "grid",
                                gridTemplateColumns: "1fr 1fr",
                                gap: "8px",
                            }}
                        >
                            <input
                                type="number"
                                inputMode="decimal"
                                placeholder="mm"
                                disabled={
                                    currentSettings.preset !== "custom" &&
                                    currentSettings.preset !== "thermal"
                                }
                                value={currentSettings.width_mm ?? ""}
                                onChange={(e) => {
                                    const width = Number(e.target.value);
                                    saveSettings(selectedPrinter, {
                                        ...currentSettings,
                                        width_mm: Number.isFinite(width)
                                            ? width
                                            : undefined,
                                    });
                                }}
                                style={{
                                    height: "40px",
                                    borderRadius: "12px",
                                    padding: "0 12px",
                                    border: "1px solid rgba(255,255,255,0.10)",
                                    background:
                                        currentSettings.preset === "custom" ||
                                            currentSettings.preset === "thermal"
                                            ? "#0f0f0f"
                                            : "rgba(255,255,255,0.04)",
                                    color: "#ffffff",
                                    outline: "none",
                                }}
                            />
                            <input
                                type="number"
                                inputMode="decimal"
                                placeholder="mm"
                                disabled={
                                    currentSettings.preset !== "custom" &&
                                    currentSettings.preset !== "thermal"
                                }
                                value={currentSettings.height_mm ?? ""}
                                onChange={(e) => {
                                    const height = Number(e.target.value);
                                    saveSettings(selectedPrinter, {
                                        ...currentSettings,
                                        height_mm: Number.isFinite(height)
                                            ? height
                                            : undefined,
                                    });
                                }}
                                style={{
                                    height: "40px",
                                    borderRadius: "12px",
                                    padding: "0 12px",
                                    border: "1px solid rgba(255,255,255,0.10)",
                                    background:
                                        currentSettings.preset === "custom" ||
                                            currentSettings.preset === "thermal"
                                            ? "#0f0f0f"
                                            : "rgba(255,255,255,0.04)",
                                    color: "#ffffff",
                                    outline: "none",
                                }}
                            />
                        </div>

                        <div
                            style={{
                                gridColumn: "span 2",
                                fontSize: "11px",
                                color: "rgba(255,255,255,0.5)",
                            }}
                        >
                            {t("printers.paperHint")}
                        </div>
                    </div>
                )}
            </motion.div>

            {/* Network Discovery Card */}
            <motion.div className="card" variants={itemVariants}>
                <div className="card-header">
                    <h3 className="card-title">
                        {t("printers.networkDiscovery") || "Descubrimiento de Red"}
                    </h3>
                    <motion.button
                        className="btn btn-primary"
                        onClick={handleScanNetwork}
                        disabled={isScanning}
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        style={{
                            background: isScanning
                                ? "linear-gradient(135deg, #64748b 0%, #475569 100%)"
                                : "linear-gradient(135deg, #88FCA4 0%, #60D98D 100%)",
                            color: "#0a0a0a",
                        }}
                    >
                        <SimpleIcon
                            icon={isScanning ? "loading" : "refresh"}
                            size={18}
                            color="#0a0a0a"
                            className={isScanning ? "animate-spin" : ""}
                        />
                        {isScanning
                            ? t("printers.scanning") || "Escaneando..."
                            : t("printers.scanNetwork") || "Escanear Red"}
                    </motion.button>
                </div>

                {localIp && (
                    <div
                        style={{
                            padding: "12px",
                            background: "rgba(136,252,164,0.08)",
                            borderRadius: "8px",
                            marginBottom: "16px",
                            display: "flex",
                            alignItems: "center",
                            gap: "8px",
                        }}
                    >
                        <SimpleIcon
                            icon="globe"
                            size={16}
                            color="#88FCA4"
                        />
                        <span style={{ fontSize: "13px", color: "#88FCA4" }}>
                            {t("printers.yourIp") || "Tu IP local"}: {localIp}
                        </span>
                    </div>
                )}

                {showNetworkDiscovery && (
                    <AnimatePresence>
                        {networkPrinters.length === 0 ? (
                            <motion.div
                                className="empty-state"
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                exit={{ opacity: 0 }}
                            >
                                <SimpleIcon
                                    icon="printer"
                                    size={32}
                                    color="#94a3b8"
                                />
                                <p>
                                    {t("printers.noPrintersFound") ||
                                        "No se encontraron impresoras en la red"}
                                </p>
                            </motion.div>
                        ) : (
                            <div
                                style={{
                                    display: "grid",
                                    gap: "8px",
                                }}
                            >
                                {networkPrinters.map((printer, index) => (
                                    <motion.div
                                        key={`${printer.ip}-${printer.port}`}
                                        initial={{ opacity: 0, x: -20 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        transition={{ delay: index * 0.05 }}
                                        style={{
                                            padding: "16px",
                                            background:
                                                "linear-gradient(180deg, #161616, #0b0b0b)",
                                            border: "1px solid rgba(136,252,164,0.2)",
                                            borderRadius: "12px",
                                            display: "flex",
                                            justifyContent: "space-between",
                                            alignItems: "center",
                                        }}
                                    >
                                        <div
                                            style={{
                                                display: "flex",
                                                flexDirection: "column",
                                                gap: "4px",
                                            }}
                                        >
                                            <span
                                                style={{
                                                    color: "#f5f5f5",
                                                    fontWeight: 500,
                                                }}
                                            >
                                                {printer.name}
                                            </span>
                                            <span
                                                style={{
                                                    fontSize: "12px",
                                                    color: "rgba(255,255,255,0.6)",
                                                }}
                                            >
                                                {printer.ip}:{printer.port} (
                                                {printer.protocol.toUpperCase()})
                                            </span>
                                        </div>
                                        <motion.button
                                            className="btn btn-sm btn-primary"
                                            onClick={() =>
                                                handleAddNetworkPrinter(printer)
                                            }
                                            whileHover={{ scale: 1.05 }}
                                            whileTap={{ scale: 0.95 }}
                                            style={{
                                                padding: "8px 16px",
                                                fontSize: "13px",
                                                background:
                                                    "linear-gradient(135deg, #88FCA4 0%, #60D98D 100%)",
                                                color: "#0a0a0a",
                                                border: "none",
                                                borderRadius: "8px",
                                                cursor: "pointer",
                                                fontWeight: 600,
                                            }}
                                        >
                                            {t("printers.addPrinter") || "Agregar"}
                                        </motion.button>
                                    </motion.div>
                                ))}
                            </div>
                        )}
                    </AnimatePresence>
                )}
            </motion.div>

            {/* Actions Card */}
            <motion.div className="card" variants={itemVariants}>
                <div className="card-header">
                    <h3 className="card-title">{t("account.action")}</h3>
                </div>

                <div className="actions-grid">
                    <motion.button
                        className="action-btn"
                        onClick={handleTestPrint}
                        disabled={!selectedPrinter}
                        whileHover={{ scale: 1.02, y: -2 }}
                        whileTap={{ scale: 0.98 }}
                    >
                        <div className="action-icon">
                            <SimpleIcon
                                icon="printer"
                                size={28}
                                color="#88FCA4"
                            />
                        </div>
                        <span className="action-label">
                            {t("printers.testPrint")}
                        </span>
                    </motion.button>

                    <motion.button
                        className="action-btn"
                        onClick={handleCut}
                        disabled={!selectedPrinter}
                        whileHover={{ scale: 1.02, y: -2 }}
                        whileTap={{ scale: 0.98 }}
                    >
                        <div className="action-icon cut">
                            <svg
                                width="28"
                                height="28"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="#F8FD67"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                            >
                                <circle cx="6" cy="6" r="3" />
                                <circle cx="6" cy="18" r="3" />
                                <line x1="20" y1="4" x2="8.12" y2="15.88" />
                                <line x1="14.47" y1="14.48" x2="20" y2="20" />
                                <line x1="8.12" y1="8.12" x2="12" y2="12" />
                            </svg>
                        </div>
                        <span className="action-label">
                            {t("printers.cut")}
                        </span>
                    </motion.button>
                </div>
            </motion.div>
        </motion.div>
    );
}
