import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import es from './locales/es.json';
import en from './locales/en.json';
import fr from './locales/fr.json';

// Obtener idioma guardado o usar español por defecto
const savedLanguage = localStorage.getItem('isiprint-language') || 'es';

i18n
  .use(initReactI18next)
  .init({
    resources: {
      es: { translation: es },
      en: { translation: en },
      fr: { translation: fr },
    },
    lng: savedLanguage,
    fallbackLng: 'es',
    interpolation: {
      escapeValue: false,
    },
  });

// Función para cambiar idioma y guardarlo
export const changeLanguage = (lang: string) => {
  i18n.changeLanguage(lang);
  localStorage.setItem('isiprint-language', lang);
};

export default i18n;
