import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { login } from '../api';
import { changeLanguage } from '../i18n';
import { StaticLogo, AnimatedLogo } from './AnimatedLogo';
import { SimpleIcon } from './LordIcon';
import type { AuthState } from '../types';

interface LoginScreenProps {
  onLoginSuccess: (authState: AuthState) => void;
}

export function LoginScreen({ onLoginSuccess }: LoginScreenProps) {
  const { t, i18n } = useTranslation();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPassword, setShowPassword] = useState(false);
  const [gridMode, setGridMode] = useState<'rest' | 'mergeTop' | 'mergeBottom'>('rest');

  useEffect(() => {
    const sequence: Array<'rest' | 'mergeTop' | 'mergeBottom'> = ['rest', 'mergeTop', 'rest', 'mergeBottom'];
    let index = 0;

    const id = window.setInterval(() => {
      index = (index + 1) % sequence.length;
      setGridMode(sequence[index]);
    }, 3200);

    return () => window.clearInterval(id);
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);

    try {
      const response = await login(email, password);
      if (response.success && response.data) {
        onLoginSuccess(response.data);
      } else {
        setError(response.error || t('auth.invalidCredentials'));
        setIsLoading(false);
      }
    } catch (err) {
      setError(t('auth.loginError'));
      setIsLoading(false);
    }
  };

  const handleLanguageChange = (lang: string) => {
    changeLanguage(lang);
  };

  const languages = [
    { code: 'es', flag: '🇪🇸' },
    { code: 'en', flag: '🇺🇸' },
    { code: 'fr', flag: '🇫🇷' },
  ];

  // Full screen loading overlay
  if (isLoading) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: '#0a0a0a',
          zIndex: 9999,
        }}
      >
        <AnimatedLogo 
          size={120} 
          color="#88FCA4" 
          isAnimated={true}
          showText={true}
          text={t('auth.loggingIn')}
          subText={email}
        />
      </motion.div>
    );
  }

  return (
    <div style={{
      display: 'flex',
      height: '100vh',
      overflow: 'hidden',
      backgroundColor: '#0a0a0a',
      color: '#ffffff',
      fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
    }}>
      {/* Left Panel - Login Form */}
      <div style={{
        width: '50%',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        padding: '28px',
        position: 'relative',
      }}>
        {/* Language Selector - Top Right of Left Panel */}
        <div style={{
          position: 'absolute',
          top: '24px',
          right: '24px',
          display: 'flex',
          gap: '8px',
        }}>
          <select
            value={i18n.language}
            onChange={(e) => handleLanguageChange(e.target.value)}
            style={{
              height: '36px',
              borderRadius: '10px',
              padding: '0 12px',
              border: '1px solid rgba(255,255,255,0.15)',
              background: 'rgba(255,255,255,0.04)',
              color: '#ffffff',
              cursor: 'pointer',
              fontSize: '13px',
              outline: 'none',
              appearance: 'none',
              WebkitAppearance: 'none',
            }}
          >
            {languages.map((lang) => (
              <option key={lang.code} value={lang.code} style={{ background: '#0a0a0a', color: '#ffffff' }}>
                {lang.flag} {lang.code.toUpperCase()}
              </option>
            ))}
          </select>
        </div>

        {/* Logo */}
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          style={{ marginBottom: '24px' }}
        >
          <div style={{
            width: '56px',
            height: '56px',
            border: '2px solid rgba(136, 252, 164, 0.3)',
            borderRadius: '14px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'rgba(136, 252, 164, 0.05)',
          }}>
            <StaticLogo size={36} color="#88FCA4" />
          </div>
        </motion.div>

        {/* Badge */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.1 }}
          style={{
            padding: '8px 16px',
            border: '1px solid rgba(255,255,255,0.15)',
            borderRadius: '20px',
            fontSize: '12px',
            color: 'rgba(255,255,255,0.7)',
            marginBottom: '24px',
            letterSpacing: '1px',
          }}
        >
          {t('auth.welcomeBack')}
        </motion.div>

        {/* Title */}
        <motion.h1
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.15 }}
          style={{
            fontSize: '32px',
            fontWeight: 600,
            marginBottom: '12px',
            color: '#ffffff',
          }}
        >
          {t('auth.loginTitle')}
        </motion.h1>

        {/* Subtitle */}
        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.2 }}
          style={{
            fontSize: '14px',
            color: 'rgba(255,255,255,0.5)',
            marginBottom: '28px',
          }}
        >
          {t('auth.loginSubtitle')}
        </motion.p>

        {/* Login Form */}
        <motion.form
          onSubmit={handleSubmit}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.25 }}
          style={{
            width: '100%',
            maxWidth: '320px',
            display: 'flex',
            flexDirection: 'column',
            gap: '16px',
          }}
        >
          {/* Email Input */}
          <div style={{ position: 'relative' }}>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t('auth.emailPlaceholder')}
              required
              style={{
                width: '100%',
                padding: '16px 20px',
                backgroundColor: 'transparent',
                border: '1px solid rgba(255,255,255,0.15)',
                borderRadius: '12px',
                color: '#ffffff',
                fontSize: '14px',
                outline: 'none',
                transition: 'border-color 0.2s ease',
                boxSizing: 'border-box',
              }}
              onFocus={(e) => e.target.style.borderColor = '#88FCA4'}
              onBlur={(e) => e.target.style.borderColor = 'rgba(255,255,255,0.15)'}
            />
          </div>

          {/* Password Input */}
          <div style={{ position: 'relative' }}>
            <input
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t('auth.passwordPlaceholder')}
              required
              style={{
                width: '100%',
                padding: '16px 50px 16px 20px',
                backgroundColor: 'transparent',
                border: '1px solid rgba(255,255,255,0.15)',
                borderRadius: '12px',
                color: '#ffffff',
                fontSize: '14px',
                outline: 'none',
                transition: 'border-color 0.2s ease',
                boxSizing: 'border-box',
              }}
              onFocus={(e) => e.target.style.borderColor = '#88FCA4'}
              onBlur={(e) => e.target.style.borderColor = 'rgba(255,255,255,0.15)'}
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              style={{
                position: 'absolute',
                right: '16px',
                top: '50%',
                transform: 'translateY(-50%)',
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                color: 'rgba(255,255,255,0.4)',
                padding: '4px',
              }}
            >
              {showPassword ? '👁️' : '👁️‍🗨️'}
            </button>
          </div>

          {/* Error Message */}
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              style={{
                padding: '12px 16px',
                backgroundColor: 'rgba(239, 68, 68, 0.1)',
                border: '1px solid rgba(239, 68, 68, 0.3)',
                borderRadius: '8px',
                color: '#fca5a5',
                fontSize: '13px',
              }}
            >
              {error}
            </motion.div>
          )}

          {/* Submit Button */}
          <motion.button
            type="submit"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            style={{
              width: '100%',
              padding: '16px',
              backgroundColor: '#ffffff',
              border: 'none',
              borderRadius: '12px',
              color: '#0a0a0a',
              fontSize: '14px',
              fontWeight: 600,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '8px',
              marginTop: '8px',
            }}
          >
            {t('auth.login')}
            <span style={{ fontSize: '16px' }}>→</span>
          </motion.button>
        </motion.form>
      </div>

      {/* Right Panel - Bento Grid */}
      <div style={{
        width: '50%',
        padding: '20px',
        display: 'grid',
        gridTemplateColumns: 'repeat(4, 1fr)',
        gridTemplateRows: 'repeat(4, 1fr)',
        gap: '16px',
        height: '100%',
        overflow: 'hidden',
      }}>
        {/* Large card top-left */}
        <motion.div
          layout
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 0.3, type: 'spring', stiffness: 260, damping: 24 }}
          style={{
            gridColumn: 'span 2',
            gridRow: 'span 2',
            background: 'linear-gradient(145deg, #1a1a1a 0%, #141414 100%)',
            borderRadius: '24px',
            border: '1px solid rgba(255,255,255,0.08)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            overflow: 'hidden',
          }}
        >
          <SimpleIcon icon="printer" size={80} color="rgba(255,255,255,0.1)" />
        </motion.div>

        {/* Top-right: bubble merge/split */}
        <AnimatePresence initial={false} mode="popLayout">
          {gridMode === 'mergeTop' ? (
            <motion.div
              key="top-merged"
              layout
              initial={{ opacity: 0, scale: 0.96 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.98 }}
              transition={{ type: 'spring', stiffness: 260, damping: 24 }}
              style={{
                gridColumn: '3 / span 2',
                gridRow: '1 / span 2',
                background: 'linear-gradient(135deg, rgba(136, 252, 164, 0.10) 0%, rgba(248, 253, 103, 0.06) 100%)',
                borderRadius: '24px',
                border: '1px solid rgba(136, 252, 164, 0.18)',
                position: 'relative',
                overflow: 'hidden',
              }}
            >
              <div style={{
                position: 'absolute',
                inset: 0,
                background: 'radial-gradient(600px 240px at 20% 30%, rgba(136, 252, 164, 0.18) 0%, rgba(136, 252, 164, 0) 60%), radial-gradient(520px 220px at 80% 70%, rgba(248, 253, 103, 0.14) 0%, rgba(248, 253, 103, 0) 60%)',
              }} />
              <div style={{
                position: 'relative',
                height: '100%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}>
                <StaticLogo size={56} color="rgba(255,255,255,0.18)" />
              </div>
            </motion.div>
          ) : (
            <>
              <motion.div
                key="tr-1"
                layout
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ delay: 0.35, type: 'spring', stiffness: 260, damping: 24 }}
                style={{
                  gridColumn: 3,
                  gridRow: 1,
                  background: '#1a1a1a',
                  borderRadius: '16px',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              />
              <motion.div
                key="tr-2"
                layout
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ delay: 0.4, type: 'spring', stiffness: 260, damping: 24 }}
                style={{
                  gridColumn: 4,
                  gridRow: 1,
                  background: '#1a1a1a',
                  borderRadius: '16px',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              />
              <motion.div
                key="tr-3"
                layout
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ delay: 0.45, type: 'spring', stiffness: 260, damping: 24 }}
                style={{
                  gridColumn: 3,
                  gridRow: 2,
                  background: '#1a1a1a',
                  borderRadius: '16px',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              />
              <motion.div
                key="tr-4"
                layout
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ delay: 0.5, type: 'spring', stiffness: 260, damping: 24 }}
                style={{
                  gridColumn: 4,
                  gridRow: 2,
                  background: '#1a1a1a',
                  borderRadius: '16px',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              />
            </>
          )}
        </AnimatePresence>

        {/* Feature card - Yellow */}
        <motion.div
          layout
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 0.55, type: 'spring', stiffness: 260, damping: 24 }}
          style={{
            gridColumn: 'span 2',
            gridRow: 3,
            background: '#F8FD67',
            borderRadius: '24px',
            padding: '24px',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <h3 style={{ 
              color: '#0a0a0a', 
              fontSize: '20px', 
              fontWeight: 700,
              marginBottom: '8px',
            }}>
              {t('auth.featureTitle1')}
            </h3>
            <p style={{ 
              color: 'rgba(10,10,10,0.7)', 
              fontSize: '13px',
              lineHeight: 1.4,
            }}>
              {t('auth.featureDesc1')}
            </p>
          </div>
          <div style={{
            width: '40px',
            height: '40px',
            background: '#0a0a0a',
            borderRadius: '12px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}>
            <SimpleIcon icon="printer" size={20} color="#F8FD67" />
          </div>
        </motion.div>

        {/* Small card */}
        <motion.div
          layout
          initial={{ opacity: 0, y: 20 }}
          animate={{
            opacity: 1,
            y: 0,
          }}
          transition={{ delay: 0.6, type: 'spring', stiffness: 260, damping: 24 }}
          style={{
            gridColumn: 3,
            gridRow: 3,
            background: '#1a1a1a',
            borderRadius: '16px',
            border: '1px solid rgba(255,255,255,0.08)',
          }}
        />

        {/* Feature card - Green */}
        <motion.div
          layout
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 0.65, type: 'spring', stiffness: 260, damping: 24 }}
          style={{
            gridColumn: 4,
            gridRow: 3,
            background: '#88FCA4',
            borderRadius: '24px',
            padding: '24px',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <h3 style={{
              color: '#0a0a0a',
              fontSize: '18px',
              fontWeight: 800,
              marginBottom: '8px',
              letterSpacing: '0.2px',
            }}>
              Todo listo
            </h3>
            <p style={{
              color: 'rgba(10,10,10,0.7)',
              fontSize: '13px',
              lineHeight: 1.4,
              margin: 0,
            }}>
              Conectado y preparado para imprimir.
            </p>
          </div>
          <div style={{
            width: '36px',
            height: '36px',
            background: '#0a0a0a',
            borderRadius: '10px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}>
            <SimpleIcon icon="check" size={18} color="#88FCA4" />
          </div>
        </motion.div>

        {/* Bottom row: bubble merge/split */}
        <AnimatePresence initial={false} mode="popLayout">
          {gridMode === 'mergeBottom' ? (
            <motion.div
              key="bottom-merged"
              layout
              initial={{ opacity: 0, scale: 0.98 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.98 }}
              transition={{ type: 'spring', stiffness: 260, damping: 24 }}
              style={{
                gridColumn: '1 / span 2',
                gridRow: 4,
                background: 'linear-gradient(135deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.03) 100%)',
                borderRadius: '16px',
                border: '1px solid rgba(255,255,255,0.08)',
              }}
            />
          ) : (
            <>
              <motion.div
                key="b-1"
                layout
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ delay: 0.7, type: 'spring', stiffness: 260, damping: 24 }}
                style={{
                  gridColumn: 1,
                  gridRow: 4,
                  background: '#1a1a1a',
                  borderRadius: '16px',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              />
              <motion.div
                key="b-2"
                layout
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ delay: 0.75, type: 'spring', stiffness: 260, damping: 24 }}
                style={{
                  gridColumn: 2,
                  gridRow: 4,
                  background: '#1a1a1a',
                  borderRadius: '16px',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              />
            </>
          )}
        </AnimatePresence>

        <motion.div
          layout
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.8, type: 'spring', stiffness: 260, damping: 24 }}
          style={{
            gridColumn: '3 / span 2',
            gridRow: 4,
            background: '#1a1a1a',
            borderRadius: '16px',
            border: '1px solid rgba(255,255,255,0.08)',
          }}
        />
      </div>

      {/* CSS for spin animation */}
      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
        input::placeholder {
          color: rgba(255,255,255,0.3);
        }
      `}</style>
    </div>
  );
}
