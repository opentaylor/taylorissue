import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhCN from "./locales/zh-CN.json";
import enUS from "./locales/en-US.json";

const savedLang = localStorage.getItem("website-lang");

i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { translation: zhCN },
    "en-US": { translation: enUS },
  },
  lng: savedLang || "en-US",
  fallbackLng: "en-US",
  interpolation: { escapeValue: false },
});

i18n.on("languageChanged", (lng) => {
  localStorage.setItem("website-lang", lng);
});

export default i18n;
