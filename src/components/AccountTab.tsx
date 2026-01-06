import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { getLicencias } from '../api';
import { SimpleIcon } from './LordIcon';
import { AnimatedLogo } from './AnimatedLogo';
import type { AuthState, LicenciaProducto } from '../types';
import './AccountTab.css';

interface AccountTabProps {
  authState: AuthState;
}

export function AccountTab({ authState }: AccountTabProps) {
  const { t } = useTranslation();
  const [licencias, setLicencias] = useState<LicenciaProducto[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const loadLicencias = async () => {
      try {
        setIsLoading(true);
        const response = await getLicencias();
        if (response.success && response.data) {
          setLicencias(response.data);
        }
      } catch (err) {
        console.error('Error loading licenses:', err);
      } finally {
        setIsLoading(false);
      }
    };
    loadLicencias();
  }, []);

  const getStatusBadge = (licencia: LicenciaProducto) => {
    if (licencia.state !== 'ACTIVADO') {
      return <span className="badge badge-error" style={{ background: 'rgba(248,253,103,0.2)', color: '#F8FD67', padding: '4px 12px', borderRadius: '20px', fontSize: '12px' }}>{t('status.inactive')}</span>;
    }
    
    // Check expiration - handle undefined fecha_vencimiento
    if (!licencia.fecha_vencimiento) {
      return <span className="badge badge-success" style={{ background: 'rgba(136,252,164,0.2)', color: '#88FCA4', padding: '4px 12px', borderRadius: '20px', fontSize: '12px' }}>{t('status.active')}</span>;
    }
    
    try {
      const parts = licencia.fecha_vencimiento.split(' ')[0].split('/');
      const expDate = new Date(parseInt(parts[2]), parseInt(parts[1]) - 1, parseInt(parts[0]));
      const now = new Date();
      
      if (expDate < now) {
        return <span className="badge badge-error" style={{ background: 'rgba(248,253,103,0.2)', color: '#F8FD67', padding: '4px 12px', borderRadius: '20px', fontSize: '12px' }}>{t('status.expired')}</span>;
      }
    } catch (e) {
      console.error('Error parsing date:', licencia.fecha_vencimiento, e);
    }
    
    return <span className="badge badge-success" style={{ background: 'rgba(136,252,164,0.2)', color: '#88FCA4', padding: '4px 12px', borderRadius: '20px', fontSize: '12px' }}>{t('status.active')}</span>;
  };

  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: { staggerChildren: 0.1 }
    }
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: { opacity: 1, y: 0 }
  };

  return (
    <motion.div 
      className="account-tab"
      style={{ backgroundColor: '#0a0a0a', color: '#ffffff', minHeight: '100%', display: 'flex', flexDirection: 'column', gap: '24px' }}
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      {/* Header */}
      <motion.div className="page-header" style={{ marginBottom: '8px' }} variants={itemVariants}>
        <h1 style={{ fontSize: '28px', fontWeight: 700, color: '#ffffff', marginBottom: '8px' }}>{t('account.title')}</h1>
        <p style={{ color: 'rgba(255,255,255,0.7)', fontSize: '14px' }}>{t('auth.loggedInAs')}: <strong style={{ color: '#88FCA4' }}>{authState.email}</strong></p>
      </motion.div>

      {/* Welcome Card */}
      <motion.div 
        className="card welcome-card" 
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '20px',
          background: 'linear-gradient(135deg, rgba(136, 252, 164, 0.1) 0%, rgba(248, 253, 103, 0.05) 100%)',
          border: '1px solid rgba(136, 252, 164, 0.2)',
          borderRadius: '16px',
          padding: '24px',
        }}
        variants={itemVariants}
      >
        <div className="welcome-icon" style={{ 
          width: '80px', 
          height: '80px', 
          background: 'rgba(136, 252, 164, 0.1)', 
          borderRadius: '20px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center'
        }}>
          <SimpleIcon icon="user" size={48} color="#88FCA4" />
        </div>
        <div className="welcome-content" style={{ flex: 1 }}>
          <h2 style={{ fontSize: '24px', fontWeight: 600, color: '#ffffff', marginBottom: '4px' }}>{t('auth.welcome')}!</h2>
          <p style={{ color: 'rgba(255,255,255,0.7)', fontSize: '14px' }}>{authState.email}</p>
        </div>
        <div className="status-indicator" style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '8px 16px',
          background: 'rgba(136, 252, 164, 0.1)',
          borderRadius: '20px',
          color: '#88FCA4',
          fontSize: '13px',
          fontWeight: 500,
        }}>
          <span className="status-dot" style={{ width: '8px', height: '8px', background: '#88FCA4', borderRadius: '50%' }} />
          <span>{t('status.connected')}</span>
        </div>
      </motion.div>

      {/* License Info */}
      <motion.div className="card" style={{ background: '#111111', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '16px', padding: '24px' }} variants={itemVariants}>
        <div className="card-header" style={{ marginBottom: '20px' }}>
          <h3 className="card-title" style={{ fontSize: '18px', fontWeight: 600, color: '#ffffff' }}>{t('account.licenseInfo')}</h3>
        </div>

        {isLoading ? (
          <div className="loading-state" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: '16px', padding: '60px' }}>
            <AnimatedLogo size={80} color="#88FCA4" isAnimated={true} />
            <span style={{ color: 'rgba(255,255,255,0.7)' }}>{t('common.loading')}</span>
          </div>
        ) : licencias.length === 0 ? (
          <div className="empty-state" style={{ textAlign: 'center', padding: '40px', color: 'rgba(255,255,255,0.5)' }}>
            <div className="empty-state-icon" style={{ marginBottom: '16px' }}>
              <SimpleIcon icon="error" size={32} color="#94a3b8" />
            </div>
            <p>{t('account.noLicense')}</p>
          </div>
        ) : (
          <div className="license-grid" style={{ display: 'grid', gap: '16px' }}>
            {licencias.map((licencia) => (
              <motion.div 
                key={licencia._id} 
                className="license-card"
                style={{
                  background: '#1a1a1a',
                  border: '1px solid rgba(136, 252, 164, 0.1)',
                  borderRadius: '12px',
                  padding: '20px',
                }}
                whileHover={{ scale: 1.02 }}
                transition={{ type: 'spring', stiffness: 400 }}
              >
                <div className="license-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                  <div className="license-type" style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#ffffff' }}>
                    <SimpleIcon icon="check" size={20} color="#88FCA4" />
                    <span style={{ fontWeight: 500 }}>{licencia.tipo_producto || 'Licencia'}</span>
                  </div>
                  {getStatusBadge(licencia)}
                </div>
                
                <div className="license-details" style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '16px' }}>
                  <div className="detail-row" style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span className="detail-label" style={{ color: 'rgba(255,255,255,0.5)' }}>{t('account.maxConnections')}</span>
                    <span className="detail-value" style={{ color: '#ffffff', fontWeight: 500 }}>{licencia.maximo_conexiones || 'N/A'}</span>
                  </div>
                  <div className="detail-row" style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span className="detail-label" style={{ color: 'rgba(255,255,255,0.5)' }}>{t('account.expirationDate')}</span>
                    <span className="detail-value" style={{ color: '#ffffff', fontWeight: 500 }}>{licencia.fecha_vencimiento || 'N/A'}</span>
                  </div>
                </div>

                {/* Progress bar for connections */}
                <div className="usage-section">
                  <div className="usage-header" style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px', fontSize: '13px', color: 'rgba(255,255,255,0.7)' }}>
                    <span>{t('account.printCount')}</span>
                    <span>0 / {licencia.maximo_conexiones || 0}</span>
                  </div>
                  <div className="usage-bar" style={{ height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', overflow: 'hidden' }}>
                    <div className="usage-fill" style={{ width: '0%', height: '100%', background: 'linear-gradient(90deg, #88FCA4, #F8FD67)', borderRadius: '4px' }} />
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        )}
      </motion.div>
    </motion.div>
  );
}
