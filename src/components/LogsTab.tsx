import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { getLogs, clearPrintJobs } from '../api';
import { SimpleIcon } from './LordIcon';
import { AnimatedLogo } from './AnimatedLogo';
import type { LogEntry } from '../types';
import './LogsTab.css';

export function LogsTab() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isClearing, setIsClearing] = useState(false);

  useEffect(() => {
    loadLogs();
    // Auto refresh every 5 seconds
    const interval = setInterval(loadLogs, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadLogs = async () => {
    try {
      if (!isLoading) setIsLoading(false); // Don't show loading on refresh
      const logsList = await getLogs();
      setLogs(logsList.reverse()); // Newest first
    } catch (err) {
      console.error('Error loading logs:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClear = async () => {
    if (!confirm(t('logs.clearConfirm'))) return;
    
    try {
      setIsClearing(true);
      await clearPrintJobs();
      setLogs([]);
    } catch (err) {
      console.error('Error clearing logs:', err);
    } finally {
      setIsClearing(false);
    }
  };

  const getLevelColor = (level: string) => {
    switch (level.toUpperCase()) {
      case 'ERROR':
        return '#F8FD67';
      case 'WARNING':
      case 'WARN':
        return '#F8FD67';
      case 'SUCCESS':
        return '#88FCA4';
      case 'INFO':
      default:
        return '#88FCA4';
    }
  };

  const getLevelIcon = (level: string) => {
    switch (level.toUpperCase()) {
      case 'ERROR':
        return 'error';
      case 'SUCCESS':
        return 'check';
      default:
        return 'info';
    }
  };

  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: { staggerChildren: 0.05 }
    }
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 10 },
    visible: { opacity: 1, y: 0 }
  };

  return (
    <motion.div 
      className="logs-tab"
      style={{ backgroundColor: '#0a0a0a', color: '#ffffff', minHeight: '100%' }}
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      {/* Header */}
      <motion.div className="page-header logs-header" variants={itemVariants}>
        <div className="header-content">
          <h1>{t('logs.title')}</h1>
          <p>{logs.length} {t('logs.title').toLowerCase()}</p>
        </div>
        <div className="header-actions">
          <motion.button
            className="btn btn-secondary"
            onClick={loadLogs}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <SimpleIcon icon="refresh" size={18} color="#94a3b8" />
            {t('logs.refresh')}
          </motion.button>
          <motion.button
            className="btn btn-danger"
            onClick={handleClear}
            disabled={isClearing || logs.length === 0}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            {isClearing ? (
              <SimpleIcon icon="loading" size={18} color="#F8FD67" />
            ) : (
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#F8FD67" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
            )}
            {t('logs.clear')}
          </motion.button>
        </div>
      </motion.div>

      {/* Logs List */}
      <motion.div className="card logs-card" variants={itemVariants}>
        {isLoading ? (
          <div className="loading-state" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: '16px', padding: '60px' }}>
            <AnimatedLogo size={80} color="#88FCA4" isAnimated={true} />
            <span style={{ color: 'rgba(255,255,255,0.7)' }}>{t('common.loading')}</span>
          </div>
        ) : logs.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <SimpleIcon icon="logs" size={32} color="#94a3b8" />
            </div>
            <p>{t('logs.noLogs')}</p>
          </div>
        ) : (
          <div className="logs-list">
            <AnimatePresence>
              {logs.map((log, index) => (
                <motion.div
                  key={`${log.timestamp}-${index}`}
                  className="log-item"
                  variants={itemVariants}
                  initial="hidden"
                  animate="visible"
                  exit={{ opacity: 0, x: -20 }}
                  transition={{ delay: index * 0.02 }}
                >
                  <div 
                    className="log-level-indicator" 
                    style={{ backgroundColor: getLevelColor(log.level) }}
                  />
                  <div className="log-icon">
                    <SimpleIcon 
                      icon={getLevelIcon(log.level) as any} 
                      size={18} 
                      color={getLevelColor(log.level)} 
                    />
                  </div>
                  <div className="log-content">
                    <div className="log-header">
                      <span 
                        className="log-level" 
                        style={{ color: getLevelColor(log.level) }}
                      >
                        {log.level}
                      </span>
                      <span className="log-time">{log.timestamp}</span>
                    </div>
                    <p className="log-message">{log.message}</p>
                  </div>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </motion.div>
    </motion.div>
  );
}
