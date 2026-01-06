import React, { useEffect, useRef, ReactNode } from 'react';
import lottie, { AnimationItem } from 'lottie-web';

// Definimos los iconos disponibles con sus CDN de LORDICON
const LORDICON_URLS: Record<string, string> = {
  // Auth & User
  'user': 'https://cdn.lordicon.com/dxjqoygy.json',
  'login': 'https://cdn.lordicon.com/hrjifpbq.json',
  'logout': 'https://cdn.lordicon.com/moscwhoj.json',
  'lock': 'https://cdn.lordicon.com/iqxnagtd.json',
  'unlock': 'https://cdn.lordicon.com/zpxybbhl.json',
  
  // Printers & Documents
  'printer': 'https://cdn.lordicon.com/vufjamqa.json',
  'document': 'https://cdn.lordicon.com/wxnxiano.json',
  'folder': 'https://cdn.lordicon.com/lsrcesku.json',
  'check': 'https://cdn.lordicon.com/oqdmuxru.json',
  'checkmark': 'https://cdn.lordicon.com/yqzmiobz.json',
  
  // Actions
  'refresh': 'https://cdn.lordicon.com/xjovhxra.json',
  'settings': 'https://cdn.lordicon.com/lecprnjb.json',
  'trash': 'https://cdn.lordicon.com/skkahier.json',
  'cut': 'https://cdn.lordicon.com/wloilxuq.json',
  
  // Status & Alerts
  'success': 'https://cdn.lordicon.com/lupuorrc.json',
  'error': 'https://cdn.lordicon.com/tdrtiskw.json',
  'warning': 'https://cdn.lordicon.com/vihyezfv.json',
  'info': 'https://cdn.lordicon.com/nocovwne.json',
  
  // Logs & Data
  'logs': 'https://cdn.lordicon.com/nocovwne.json',
  'list': 'https://cdn.lordicon.com/ynwbvguu.json',
  'chart': 'https://cdn.lordicon.com/gqdnbnwt.json',
  
  // Misc
  'loading': 'https://cdn.lordicon.com/xjovhxra.json',
  'globe': 'https://cdn.lordicon.com/osuxyevn.json',
  'email': 'https://cdn.lordicon.com/rhvddzym.json',
  'key': 'https://cdn.lordicon.com/ftndcppn.json',
};

interface LordIconProps {
  icon: keyof typeof LORDICON_URLS | string;
  size?: number;
  colors?: {
    primary?: string;
    secondary?: string;
  };
  trigger?: 'hover' | 'click' | 'loop' | 'loop-on-hover' | 'morph' | 'boomerang';
  className?: string;
  style?: React.CSSProperties;
}

export function LordIcon({
  icon,
  size = 48,
  colors: _colors = { primary: '#6366f1', secondary: '#a5b4fc' },
  trigger = 'hover',
  className = '',
  style = {},
}: LordIconProps) {
  // colors reserved for future use
  void _colors;
  const containerRef = useRef<HTMLDivElement>(null);
  const animationRef = useRef<AnimationItem | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const iconUrl = LORDICON_URLS[icon] || icon;

    // Cargar animación con lottie-web
    const loadAnimation = async () => {
      try {
        const response = await fetch(iconUrl);
        const animationData = await response.json();

        if (containerRef.current) {
          animationRef.current = lottie.loadAnimation({
            container: containerRef.current,
            renderer: 'svg',
            loop: trigger === 'loop',
            autoplay: trigger === 'loop',
            animationData,
          });

          // Aplicar colores
          if (animationRef.current) {
            const svg = containerRef.current.querySelector('svg');
            if (svg) {
              svg.style.width = `${size}px`;
              svg.style.height = `${size}px`;
            }
          }
        }
      } catch (error) {
        console.error('Error loading LordIcon:', error);
      }
    };

    loadAnimation();

    return () => {
      if (animationRef.current) {
        animationRef.current.destroy();
      }
    };
  }, [icon, size, trigger]);

  const handleMouseEnter = () => {
    if ((trigger === 'hover' || trigger === 'loop-on-hover') && animationRef.current) {
      animationRef.current.goToAndPlay(0);
    }
  };

  const handleClick = () => {
    if (trigger === 'click' && animationRef.current) {
      animationRef.current.goToAndPlay(0);
    }
  };

  return (
    <div
      ref={containerRef}
      className={`lord-icon ${className}`}
      style={{
        width: size,
        height: size,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        ...style,
      }}
      onMouseEnter={handleMouseEnter}
      onClick={handleClick}
    />
  );
}

// Componente simplificado con SVG estáticos animados con CSS
// Para mejor rendimiento cuando no necesitamos animaciones Lottie complejas
interface SimpleIconProps {
  icon: 'user' | 'printer' | 'logs' | 'logout' | 'globe' | 'check' | 'error' | 'loading' | 'refresh';
  size?: number;
  color?: string;
  className?: string;
}

export function SimpleIcon({ icon, size = 24, color = '#6366f1', className = '' }: SimpleIconProps) {
  const icons: Record<string, ReactNode> = {
    user: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
        <circle cx="12" cy="7" r="4" />
      </svg>
    ),
    printer: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="6 9 6 2 18 2 18 9" />
        <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
        <rect x="6" y="14" width="12" height="8" />
      </svg>
    ),
    logs: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <line x1="8" y1="6" x2="21" y2="6" />
        <line x1="8" y1="12" x2="21" y2="12" />
        <line x1="8" y1="18" x2="21" y2="18" />
        <line x1="3" y1="6" x2="3.01" y2="6" />
        <line x1="3" y1="12" x2="3.01" y2="12" />
        <line x1="3" y1="18" x2="3.01" y2="18" />
      </svg>
    ),
    logout: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
        <polyline points="16 17 21 12 16 7" />
        <line x1="21" y1="12" x2="9" y2="12" />
      </svg>
    ),
    globe: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
      </svg>
    ),
    check: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="20 6 9 17 4 12" />
      </svg>
    ),
    error: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <circle cx="12" cy="12" r="10" />
        <line x1="15" y1="9" x2="9" y2="15" />
        <line x1="9" y1="9" x2="15" y2="15" />
      </svg>
    ),
    loading: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
        <line x1="12" y1="2" x2="12" y2="6" />
        <line x1="12" y1="18" x2="12" y2="22" />
        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76" />
        <line x1="16.24" y1="16.24" x2="19.07" y2="19.07" />
        <line x1="2" y1="12" x2="6" y2="12" />
        <line x1="18" y1="12" x2="22" y2="12" />
        <line x1="4.93" y1="19.07" x2="7.76" y2="16.24" />
        <line x1="16.24" y1="7.76" x2="19.07" y2="4.93" />
      </svg>
    ),
    refresh: (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="23 4 23 10 17 10" />
        <polyline points="1 20 1 14 7 14" />
        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
      </svg>
    ),
  };

  return (
    <span className={`simple-icon ${className}`} style={{ display: 'inline-flex', alignItems: 'center' }}>
      {icons[icon] || null}
    </span>
  );
}
