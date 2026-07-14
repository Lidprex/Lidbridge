'use client'

import { useState, useEffect, createContext, useContext } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { DashboardUI } from './components/dashboard-ui'
interface User {
  id: number
  github_id: string
  email: string
  name: string
  avatar_url: string
}

interface CleanProgress {
  phase: string
  current: number
  total: number
  current_file: string
  percentage: number
  bytes_copied: number
  deleted_count: number
}

interface ScanResult {
  total_files: number
  clean_files: number
  skipped_dirs: number
  skipped_files: number
  total_size: number
  clean_size: number
  project_type: string
  skippable: Record<string, number>
  secrets_count: number
  secret_matches: string[]
  secret_suggestions: string[]
}

interface CleanResult {
  success: boolean
  cleaned_path: string
  copied_files: number
  skipped_files: number
  deleted_items: string[]
  warnings: string[]
  total_size_bytes: number
  scan_result: ScanResult
}

interface RepoConfig {
  name: string
  description: string
  is_private: boolean
  include_images: boolean
  create_readme: boolean
  license_template: string
  repo_type: string
}

interface ProgressState {
  percentage: number
  message: string
}

type TranslationKey = 
  | 'welcome' | 'subtitle' | 'connect_github' | 'logout'
  | 'step1' | 'step2' | 'step3' | 'browse_folder' | 'no_folder'
  | 'files_to_remove' | 'clean_project' | 'cleaning' | 'push_github'
  | 'create_repo' | 'repo_name' | 'description' | 'visibility' | 'public' | 'private'
  | 'include_images' | 'create_readme' | 'target_folder' | 'browse_target_folder' | 'no_target_folder' | 'cancel' | 'create_push'
  | 'analyzing' | 'cleaning_complete' | 'push_complete' | 'success' | 'error'
  | 'ai_analysis' | 'ai_analysis_title' | 'ai_analysis_title_sub' | 'ai_analysis_pro'
  | 'about' | 'about_title' | 'created_by' | 'parent_company' | 'the_lab'
  | 'lead_developer' | 'organization' | 'instagram' | 'video_editor'
  | 'history' | 'history_title' | 'created_at' | 'owner' | 'your_repositories'
  | 'language' | 'scanning' | 'copying' | 'files_copied' | 'files_deleted'
  | 'security_warning'

type Translations = {
  [key in TranslationKey]: string
}

const translations: Record<string, Translations> = {
  en: {
    welcome: 'Welcome to LidBridge',
    subtitle: 'Clean your projects and push to GitHub in seconds',
    connect_github: 'Connect GitHub',
    logout: 'Logout',
    step1: 'Step 1: Select Project',
    step2: 'Step 2: Clean Project',
    step3: 'Step 3: Push to GitHub',
    browse_folder: 'Browse Folder',
    no_folder: 'No folder selected',
    files_to_remove: 'Files to remove:',
    clean_project: 'Clean Project',
    cleaning: 'Cleaning...',
    push_github: 'Push to GitHub',
    create_repo: 'Create GitHub Repository',
    repo_name: 'Repository Name',
    description: 'Description (optional)',
    visibility: 'Visibility',
    public: 'Public',
    private: 'Private',
    include_images: 'Include Images/Icons',
    create_readme: 'Create README',
    target_folder: 'Destination Folder',
    browse_target_folder: 'Browse Destination',
    no_target_folder: 'No destination selected',
    cancel: 'Cancel',
    create_push: 'Create & Push',
    analyzing: 'Analyzing project...',
    scanning: 'Scanning project...',
    copying: 'Copying files...',
    cleaning_complete: 'Cleaning complete!',
    push_complete: 'Push complete!',
    success: 'Success',
    error: 'Error',
    files_copied: 'Copied',
    files_deleted: 'Deleted',
    ai_analysis: 'AI Analysis',
    ai_analysis_title: 'AI Analysis with GitHub Copilot',
    ai_analysis_title_sub: '(Coming Soon)',
    ai_analysis_pro: 'This will be a Pro feature due to API costs.',
    about: 'About',
    about_title: 'About LidBridge',
    created_by: 'Created By',
    parent_company: 'Parent Company',
    the_lab: 'The Lab',
    lead_developer: 'Lead Developer',
    organization: 'Organization',
    instagram: 'Instagram',
    video_editor: 'Video Editor',
    history: 'Repo History',
    history_title: 'Created Repository History',
    created_at: 'Created at',
    owner: 'Owner',
    your_repositories: 'My Repositories',
    language: 'Language',
    security_warning: 'Make sure you downloaded this app from the official source. Download only from: GitHub Repository: github.com/Lidprex/Lidbridge, GitHub Releases: github.com/Lidprex/Lidbridge/releases, Official Website: lidbridge.onrender.com. If you did not download from these sources, do NOT log in.',
  },
  ar: {
    welcome: 'مرحباً بك في LidBridge',
    subtitle: 'نظف مشاريعك وادفعها إلى GitHub في ثوانٍ',
    connect_github: 'تواصل مع GitHub',
    logout: 'تسجيل الخروج',
    step1: 'الخطوة 1: اختر المشروع',
    step2: 'الخطوة 2: نظف المشروع',
    step3: 'الخطوة 3: ادفع إلى GitHub',
    browse_folder: 'تصفح المجلد',
    no_folder: 'لم يتم اختيار مجلد',
    files_to_remove: 'الملفات المراد إزالتها:',
    clean_project: 'نظف المشروع',
    cleaning: 'جاري التنظيف...',
    push_github: 'ادفع إلى GitHub',
    create_repo: 'إنشاء مستودع GitHub',
    repo_name: 'اسم المستودع',
    description: 'الوصف (اختياري)',
    visibility: 'الرؤية',
    public: 'عام',
    private: 'خاص',
    include_images: 'تضمين الصور/الأيقونات',
    create_readme: 'إنشاء README',
    target_folder: 'مجلد الوجهة',
    browse_target_folder: 'تصفح مكان الوجهة',
    no_target_folder: 'لم يتم اختيار وجهة',
    cancel: 'إلغاء',
    create_push: 'إنشاء ودفع',
    analyzing: 'جاري تحليل المشروع...',
    scanning: 'جاري فحص المشروع...',
    copying: 'جاري نسخ الملفات...',
    cleaning_complete: 'اكتمل التنظيف!',
    push_complete: 'اكتمل الدفع!',
    success: 'نجاح',
    error: 'خطأ',
    files_copied: 'تم النسخ',
    files_deleted: 'تم الحذف',
    ai_analysis: 'تحليل الذكاء الاصطناعي',
    ai_analysis_title: 'تحليل الذكاء الاصطناعي مع GitHub Copilot',
    ai_analysis_title_sub: '(قريباً)',
    ai_analysis_pro: 'ميزة مدفوعة بسبب تكاليف API',
    about: 'حول',
    about_title: 'حول LidBridge',
    created_by: 'صنع بواسطة',
    parent_company: 'الشركة الأم',
    the_lab: 'المختبر',
    lead_developer: 'المطور الرئيسي',
    organization: 'المنظمة',
    instagram: 'إنستغرام',
    video_editor: 'محرر فيديو',
    history: 'سجل المستودعات',
    history_title: 'المستودعات التي أنشأها التطبيق',
    created_at: 'أنشئ في',
    owner: 'المالك',
    your_repositories: 'مستودعاتي',
    language: 'اللغة',
    security_warning: 'تأكد من أنك قمت بتنزيل هذا التطبيق من المصدر الرسمي. قم بالتنزيل فقط من: مستودع GitHub: github.com/Lidprex/Lidbridge, إصدارات GitHub: github.com/Lidprex/Lidbridge/releases, الموقع الرسمي: lidbridge.onrender.com. إذا لم تقم بالتنزيل من هذه المصادر، لا تقم بتسجيل الدخول.',
  },
  fr: {
    welcome: 'Bienvenue sur LidBridge',
    subtitle: 'Nettoyez vos projets et poussez-les sur GitHub',
    connect_github: 'Connecter GitHub',
    logout: 'Déconnexion',
    step1: 'Étape 1: Sélectionner le projet',
    step2: 'Étape 2: Nettoyer le projet',
    step3: 'Étape 3: Pousser vers GitHub',
    browse_folder: 'Parcourir le dossier',
    no_folder: 'Aucun dossier sélectionné',
    files_to_remove: 'Fichiers à supprimer:',
    clean_project: 'Nettoyer le projet',
    cleaning: 'Nettoyage...',
    push_github: 'Pousser vers GitHub',
    create_repo: 'Créer un dépôt GitHub',
    repo_name: 'Nom du dépôt',
    description: 'Description (optionnel)',
    visibility: 'Visibilité',
    public: 'Public',
    private: 'Privé',
    include_images: 'Inclure les images/icônes',
    create_readme: 'Créer un README',
    target_folder: 'Dossier de destination',
    browse_target_folder: 'Parcourir la destination',
    no_target_folder: 'Aucune destination sélectionnée',
    cancel: 'Annuler',
    create_push: 'Créer et pousser',
    analyzing: 'Analyse du projet...',
    scanning: 'Analyse du projet...',
    copying: 'Copie des fichiers...',
    cleaning_complete: 'Nettoyage terminé!',
    push_complete: 'Push terminé!',
    success: 'Succès',
    error: 'Erreur',
    files_copied: 'Copiés',
    files_deleted: 'Supprimés',
    ai_analysis: 'Analyse IA',
    ai_analysis_title: 'Analyse IA avec GitHub Copilot',
    ai_analysis_title_sub: '(Bientôt)',
    ai_analysis_pro: 'Fonctionnalite Pro',
    about: 'À propos',
    about_title: 'À propos de LidBridge',
    created_by: 'Créé par',
    parent_company: 'Entreprise mère',
    the_lab: 'Le Lab',
    lead_developer: 'Développeur principal',
    organization: 'Organisation',
    instagram: 'Instagram',
    video_editor: 'Monteur vidéo',
    history: 'Historique des dépôts',
    history_title: 'Historique des dépôts créés',
    created_at: 'Créé le',
    owner: 'Propriétaire',
    your_repositories: 'Mes dépôts',
    language: 'Langue',
    security_warning: "Assurez-vous d'avoir téléchargé cette application depuis la source officielle. Téléchargez uniquement depuis : Dépôt GitHub : github.com/Lidprex/Lidbridge, Versions GitHub : github.com/Lidprex/Lidbridge/releases, Site officiel : lidbridge.onrender.com. Si vous ne l'avez pas téléchargée depuis ces sources, ne vous connectez PAS.",
  },
  hi: {
    welcome: 'LidBridge में आपका स्वागत है',
    subtitle: 'अपने प्रोजेक्ट्स को साफ करें',
    connect_github: 'GitHub से कनेक्ट करें',
    logout: 'लॉगआउट',
    step1: 'चरण 1: प्रोजेक्ट चुनें',
    step2: 'चरण 2: प्रोजेक्ट साफ करें',
    step3: 'चरण 3: GitHub पर पुश करें',
    browse_folder: 'फोल्डर ब्राउज़ करें',
    no_folder: 'कोई फोल्डर नहीं चुना गया',
    files_to_remove: 'हटाने के लिए फाइलें:',
    clean_project: 'प्रोजेक्ट साफ करें',
    cleaning: 'साफ हो रहा है...',
    push_github: 'GitHub पर पुश करें',
    create_repo: 'GitHub रिपॉजिटरी बनाएं',
    repo_name: 'रिपॉजिटरी का नाम',
    description: 'विवरण (वैकल्पिक)',
    visibility: 'दृश्यता',
    public: 'सार्वजनिक',
    private: 'निजी',
    include_images: 'चित्र/आइकन शामिल करें',
    create_readme: 'README बनाएं',
    target_folder: 'निर्गमन फ़ोल्डर',
    browse_target_folder: 'निर्गमन ब्राउज़ करें',
    no_target_folder: 'कोई गंतव्य चयनित नहीं',
    cancel: 'रद्द करें',
    create_push: 'बनाएं और पुश करें',
    analyzing: 'प्रोजेक्ट का विश्लेषण...',
    scanning: 'प्रोजेक्ट स्कैन हो रहा है...',
    copying: 'फाइलें कॉपी हो रही हैं...',
    cleaning_complete: 'सफाई पूर्ण!',
    push_complete: 'पुश पूर्ण!',
    success: 'सफलता',
    error: 'त्रुटि',
    files_copied: 'कॉपी की गईं',
    files_deleted: 'हटाई गईं',
    ai_analysis: 'AI विश्लेषण',
    ai_analysis_title: 'GitHub Copilot के साथ AI विश्लेषण',
    ai_analysis_title_sub: '(जल्द आ रहा है)',
    ai_analysis_pro: 'Pro सुविधा',
    about: 'के बारे में',
    about_title: 'LidBridge के बारे में',
    created_by: 'द्वारा बनाया गया',
    parent_company: 'मूल कंपनी',
    the_lab: 'द लैब',
    lead_developer: 'प्रमुख डेवलपर',
    organization: 'संगठन',
    instagram: 'इंस्टाग्राम',
    video_editor: 'वीडियो एडिटर',
    history: 'रिपॉजिटरी इतिहास',
    history_title: 'बनाई गई रिपॉजिटरी इतिहास',
    created_at: 'निर्मित तारीख',
    owner: 'स्वामी',
    your_repositories: 'मेरे रिपॉजिटरी',
    language: 'भाषा',
    security_warning: 'सुनिश्चित करें कि आपने इस ऐप को आधिकारिक स्रोत से डाउनलोड किया है। केवल इनसे डाउनलोड करें: GitHub रिपॉजिटरी: github.com/Lidprex/Lidbridge, GitHub रिलीज़: github.com/Lidprex/Lidbridge/releases, आधिकारिक वेबसाइट: lidbridge.onrender.com। यदि आपने इन स्रोतों से डाउनलोड नहीं किया है, तो लॉग इन न करें।',
  },
  zh: {
    welcome: '欢迎使用 LidBridge',
    subtitle: '清理项目并推送到 GitHub',
    connect_github: '连接 GitHub',
    logout: '退出登录',
    step1: '步骤 1: 选择项目',
    step2: '步骤 2: 清理项目',
    step3: '步骤 3: 推送到 GitHub',
    browse_folder: '浏览文件夹',
    no_folder: '未选择文件夹',
    files_to_remove: '要删除的文件:',
    clean_project: '清理项目',
    cleaning: '清理中...',
    push_github: '推送到 GitHub',
    create_repo: '创建 GitHub 仓库',
    repo_name: '仓库名称',
    description: '描述（可选）',
    visibility: '可见性',
    public: '公开',
    private: '私有',
    include_images: '包含图片/图标',
    create_readme: '创建 README',
    target_folder: '目标文件夹',
    browse_target_folder: '浏览目标',
    no_target_folder: '未选择目标',
    cancel: '取消',
    create_push: '创建并推送',
    analyzing: '分析项目中...',
    scanning: '扫描项目中...',
    copying: '复制文件中...',
    cleaning_complete: '清理完成！',
    push_complete: '推送完成！',
    success: '成功',
    error: '错误',
    files_copied: '已复制',
    files_deleted: '已删除',
    ai_analysis: 'AI 分析',
    ai_analysis_title: 'GitHub Copilot AI 分析',
    ai_analysis_title_sub: '(即将推出)',
    ai_analysis_pro: 'Pro 功能',
    about: '关于',
    about_title: '关于 LidBridge',
    created_by: '创建者',
    parent_company: '母公司',
    the_lab: '实验室',
    lead_developer: '首席开发者',
    organization: '组织',
    instagram: 'Instagram',
    video_editor: '视频编辑',
    history: '仓库历史',
    history_title: '创建的仓库历史',
    created_at: '创建于',
    owner: '拥有者',
    your_repositories: '我的仓库',
    language: '语言',
    security_warning: '请确保您从官方来源下载了此应用。仅从以下地址下载：GitHub 仓库：github.com/Lidprex/Lidbridge，GitHub 发布版：github.com/Lidprex/Lidbridge/releases，官方网站：lidbridge.onrender.com。如果您不是从这些来源下载的，请勿登录。',
  },
  ru: {
    welcome: 'Добро пожаловать в LidBridge',
    subtitle: 'Очистите проекты и отправьте на GitHub за секунды',
    connect_github: 'Подключить GitHub',
    logout: 'Выйти',
    step1: 'Шаг 1: Выберите проект',
    step2: 'Шаг 2: Очистите проект',
    step3: 'Шаг 3: Отправьте на GitHub',
    browse_folder: 'Выбрать папку',
    no_folder: 'Папка не выбрана',
    files_to_remove: 'Файлы для удаления:',
    clean_project: 'Очистить проект',
    cleaning: 'Очистка...',
    push_github: 'Отправить на GitHub',
    create_repo: 'Создать репозиторий GitHub',
    repo_name: 'Название репозитория',
    description: 'Описание (необязательно)',
    visibility: 'Видимость',
    public: 'Публичный',
    private: 'Приватный',
    include_images: 'Включить изображения/иконки',
    create_readme: 'Создать README',
    target_folder: 'Папка назначения',
    browse_target_folder: 'Выбрать назначение',
    no_target_folder: 'Назначение не выбрано',
    cancel: 'Отмена',
    create_push: 'Создать и отправить',
    analyzing: 'Анализ проекта...',
    scanning: 'Сканирование проекта...',
    copying: 'Копирование файлов...',
    cleaning_complete: 'Очистка завершена!',
    push_complete: 'Отправка завершена!',
    success: 'Успешно',
    error: 'Ошибка',
    files_copied: 'Скопировано',
    files_deleted: 'Удалено',
    ai_analysis: 'ИИ анализ',
    ai_analysis_title: 'ИИ анализ с GitHub Copilot',
    ai_analysis_title_sub: '(Скоро)',
    ai_analysis_pro: 'Функция Pro',
    about: 'О приложении',
    about_title: 'О LidBridge',
    created_by: 'Автор',
    parent_company: 'Материнская компания',
    the_lab: 'Лаборатория',
    lead_developer: 'Главный разработчик',
    organization: 'Организация',
    instagram: 'Instagram',
    video_editor: 'Видео редактор',
    history: 'История репозиториев',
    history_title: 'История созданных репозиториев',
    created_at: 'Создано',
    owner: 'Владелец',
    your_repositories: 'Мои репозитории',
    language: 'Язык',
    security_warning: 'Убедитесь, что вы скачали это приложение из официального источника. Скачивайте только с: GitHub репозиторий: github.com/Lidprex/Lidbridge, GitHub релизы: github.com/Lidprex/Lidbridge/releases, официальный сайт: lidbridge.onrender.com. Если вы скачали не из этих источников, НЕ входите в систему.',
  },
}

interface LanguageContextType {
  lang: string
  setLang: (lang: string) => void
  t: (key: TranslationKey) => string
  isRTL: boolean
}

const LanguageContext = createContext<LanguageContextType>({
  lang: 'en',
  setLang: () => {},
  t: () => '',
  isRTL: false,
})

function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLangState] = useState('en')
  const [isRTL, setIsRTL] = useState(false)

  useEffect(() => {
    const saved = localStorage.getItem('lidbridge-lang')
    if (saved && translations[saved]) {
      setLangState(saved)
      setIsRTL(saved === 'ar')
    }
  }, [])

  const setLang = (newLang: string) => {
    setLangState(newLang)
    setIsRTL(newLang === 'ar')
    localStorage.setItem('lidbridge-lang', newLang)
  }

  const t = (key: TranslationKey): string => {
    return translations[lang]?.[key] || translations.en[key] || key
  }

  return (
    <LanguageContext.Provider value={{ lang, setLang, t, isRTL }}>
      {children}
    </LanguageContext.Provider>
  )
}

function useLanguage() {
  return useContext(LanguageContext)
}

function LanguageSwitcher() {
  const { lang, setLang } = useLanguage()
  const [isOpen, setIsOpen] = useState(false)

  const languages = [
    { code: 'en', name: 'English', flag: '🇺🇸' },
    { code: 'ar', name: 'العربية', flag: '🇸🇦' },
    { code: 'ru', name: 'Русский', flag: '🇷🇺' },
    { code: 'fr', name: 'Français', flag: '🇫🇷' },
    { code: 'hi', name: 'हिन्दी', flag: '🇮🇳' },
    { code: 'zh', name: '中文', flag: '🇨🇳' },
  ]

  const currentLang = languages.find(l => l.code === lang)

  return (
    <div className="relative">
      <button onClick={() => setIsOpen(!isOpen)} className="flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary border border-border-subtle rounded-lg hover:border-accent-primary/50">
        <span>{currentLang?.flag}</span>
        <span className="text-text-secondary text-sm">{currentLang?.name}</span>
        <svg className="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {isOpen && (
        <div className="absolute top-full mt-2 right-0 bg-bg-secondary border border-border-subtle rounded-lg shadow-xl z-50 min-w-[150px]">
          {languages.map(l => (
            <button key={l.code} onClick={() => { setLang(l.code); setIsOpen(false) }} className={`w-full flex items-center gap-2 px-4 py-2 hover:bg-bg-tertiary ${lang === l.code ? 'bg-bg-tertiary' : ''}`}>
              <span>{l.flag}</span><span className="text-text-secondary">{l.name}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

function Header({ user, onLogout, onAIAnalysis, onAbout, onHistory }: { user: User | null; onLogout: () => void; onAIAnalysis: () => void; onAbout: () => void; onHistory: () => void }) {
  const { t } = useLanguage()
  return (
    <header className="h-16 bg-bg-secondary border-b border-border-subtle flex items-center justify-between px-6">
      <div className="flex items-center gap-3">
        <img 
          src="https://res.cloudinary.com/ddqedxovk/image/upload/v1777644756/zdmst5ng01o20lam01ou.png" 
          alt="LidBridge Logo" 
          className="w-8 h-8 rounded-lg object-cover"
        />
        <h1 className="text-xl font-bold text-text-primary">LidBridge</h1>
      </div>
      <div className="flex items-center gap-3">
        <LanguageSwitcher />
        <button onClick={onHistory} className="flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary border border-border-subtle rounded-lg">
          <svg className="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7h18M3 12h18M3 17h18" />
          </svg>
          <span className="text-text-secondary text-sm">{t('history')}</span>
        </button>
        <button onClick={onAIAnalysis} className="flex items-center gap-2 px-3 py-1.5 bg-gradient-to-r from-accent-primary/20 to-accent-secondary/20 border border-accent-primary/30 rounded-lg">
          <svg className="w-4 h-4 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2" />
          </svg>
          <span className="text-accent-primary text-sm">{t('ai_analysis')}</span>
        </button>
        <button onClick={onAbout} className="flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary border border-border-subtle rounded-lg">
          <svg className="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span className="text-text-secondary text-sm">{t('about')}</span>
        </button>
        {user && (
          <div className="flex items-center gap-4 ml-2">
            <div className="flex items-center gap-2"><img src={user.avatar_url} alt={user.name} className="w-8 h-8 rounded-full" /><span className="text-text-secondary">{user.name}</span></div>
            <button onClick={onLogout} className="text-text-muted hover:text-text-primary">{t('logout')}</button>
          </div>
        )}
      </div>
    </header>
  )
}

function AuthScreen({ onLogin }: { onLogin: () => void }) {
  const { t } = useLanguage()
  const [showTokenInput, setShowTokenInput] = useState(false)
  const [token, setToken] = useState('')
  
  const handleTokenSubmit = async () => {
    if (!token.trim()) {
      alert('Please enter a valid token')
      return
    }
    try {
      await invoke('save_github_token', { token: token.trim() })
      const session = await invoke<User | null>('get_session')
      if (session) {
        window.location.reload()
      }
    } catch (err) {
      alert(`Error: ${err}`)
    }
  }
  
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="card max-w-md w-full text-center">
        <div className="w-16 h-16 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-2xl flex items-center justify-center mx-auto mb-6">
          <svg className="w-10 h-10 text-black" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </div>
        <h2 className="text-2xl font-bold text-text-primary mb-2">{t('welcome')}</h2>
        <p className="text-text-secondary mb-8">{t('subtitle')}</p>
        
        {!showTokenInput ? (
          <div className="space-y-3">
            <button onClick={onLogin} className="btn btn-primary w-full flex items-center justify-center gap-2">
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
              {t('connect_github')}
            </button>
            <div className="relative">
              <div className="absolute inset-0 flex items-center"><div className="w-full border-t border-border-subtle"></div></div>
              <div className="relative flex justify-center text-sm"><span className="px-2 bg-bg-secondary text-text-muted">Or</span></div>
            </div>
            <button onClick={() => setShowTokenInput(true)} className="btn btn-secondary w-full flex items-center justify-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.192 5.978A6 6 0 1115.5 7m0 0a2 2 0 00-2-2m2 2a2 2 0 012-2m0 0V5a2 2 0 012 2v.5" /></svg>
              Use Personal Token
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            <div>
              <label className="block text-text-secondary text-sm mb-2">GitHub Personal Access Token</label>
              <input 
                type="password" 
                value={token} 
                onChange={(e) => setToken(e.target.value)}
                placeholder="ghp_xxxxxxxxxxxx"
                className="input w-full"
              />
              <p className="text-xs text-text-muted mt-2">
                Create one at: <a href="https://github.com/settings/tokens/new" target="_blank" rel="noopener noreferrer" className="text-accent-primary hover:underline">github.com/settings/tokens/new</a>
              </p>
              <p className="text-xs text-text-muted mt-1">Required scopes: <code className="bg-bg-tertiary px-2 py-1 rounded">repo</code></p>
            </div>
            <div className="flex gap-3">
              <button onClick={() => { setShowTokenInput(false); setToken('') }} className="btn btn-secondary flex-1">Cancel</button>
              <button onClick={handleTokenSubmit} className="btn btn-primary flex-1">Login</button>
            </div>
          </div>
        )}
        <div className="mt-6 p-3 rounded-lg border border-yellow-500/30 bg-yellow-500/5">
          <div className="flex items-start gap-2">
            <svg className="w-4 h-4 text-yellow-500 mt-0.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" /></svg>
            <p className="text-xs text-yellow-500/80 leading-relaxed">{t('security_warning')}</p>
          </div>
        </div>
      </div>
    </div>
  )
}

function StepSelectProject({ selectedPath, onSelectFolder }: { selectedPath: string; onSelectFolder: () => void }) {
  const { t } = useLanguage()
  return (
    <div className="card mb-6">
      <h3 className="text-lg font-semibold text-text-primary mb-4">{t('step1')}</h3>
      <div className="flex gap-3">
        <button onClick={onSelectFolder} className="btn btn-secondary flex items-center gap-2">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
          {t('browse_folder')}
        </button>
        <div className="flex-1 bg-bg-tertiary border border-border-subtle rounded-md px-4 py-2 text-text-secondary overflow-hidden text-ellipsis">{selectedPath || t('no_folder')}</div>
      </div>
    </div>
  )
}

function StepCleanProject({ selectedPath, targetPath, onSelectTargetFolder, onClean, cleaning, cleaned, scanResult, includeImages, setIncludeImages }: { selectedPath: string; targetPath: string; onSelectTargetFolder: () => void; onClean: () => void; cleaning: boolean; cleaned: boolean; scanResult: ScanResult | null; includeImages: boolean; setIncludeImages: (include: boolean) => void }) {
  const { t } = useLanguage()
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }
  return (
    <div className="card mb-6">
      <h3 className="text-lg font-semibold text-text-primary mb-4">{t('step2')}</h3>
      {scanResult && (
        <div className="mb-4 p-4 bg-bg-tertiary rounded-lg">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
            <div><p className="text-xl font-bold text-accent-primary">{scanResult.project_type}</p><p className="text-xs text-text-muted">Project Type</p></div>
            <div><p className="text-xl font-bold text-text-primary">{scanResult.total_files}</p><p className="text-xs text-text-muted">Total Files</p></div>
            <div><p className="text-xl font-bold text-accent-secondary">{scanResult.clean_files}</p><p className="text-xs text-text-muted">Clean Files</p></div>
            <div><p className="text-xl font-bold text-text-secondary">{formatBytes(scanResult.clean_size)}</p><p className="text-xs text-text-muted">Clean Size</p></div>
          </div>
          {Object.keys(scanResult.skippable).length > 0 && (
            <div className="mb-4">
              <p className="text-text-muted text-xs mb-2">{t('files_to_remove')}</p>
              <div className="flex flex-wrap gap-2">
                {Object.entries(scanResult.skippable).sort((a, b) => b[1] - a[1]).slice(0, 6).map(([name, size]) => (<span key={name} className="px-3 py-1 bg-bg-secondary rounded-full text-xs text-text-muted">{name} ({formatBytes(size)})</span>))}
              </div>
            </div>
          )}
          {(scanResult.secrets_count || 0) > 0 && (
            <div className="rounded-lg border border-warning/40 bg-warning/10 p-4">
              <p className="text-sm font-medium text-warning">Potential secrets detected</p>
              <p className="text-xs text-text-secondary mt-1">We found {scanResult.secrets_count} potential secret-like values. Review these before publishing or pushing to GitHub.</p>
              {scanResult.secret_matches.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-2">
                  {scanResult.secret_matches.slice(0, 6).map((match) => (
                    <span key={match} className="px-3 py-1 bg-bg-secondary rounded-full text-xs text-text-muted">{match}</span>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}
      <div className="mb-4">
        <div className="flex gap-3">
          <div className="flex-1 p-4 rounded-lg border border-accent-primary bg-accent-primary/10">
            <p className="font-medium text-sm text-text-primary">Smart Clean</p>
            <p className="text-xs text-text-muted">Remove junk and keep your project structure</p>
          </div>
        </div>
      </div>
      <div className="mb-4">
        <div className="flex gap-3">
          <button onClick={onSelectTargetFolder} className="btn btn-secondary flex items-center gap-2">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
            {t('browse_target_folder')}
          </button>
          <div className="flex-1 bg-bg-tertiary border border-border-subtle rounded-md px-4 py-2 text-text-secondary overflow-hidden text-ellipsis">{targetPath || t('no_target_folder')}</div>
        </div>
      </div>
      <label className="flex items-center gap-3 cursor-pointer mb-4"><input type="checkbox" checked={includeImages} onChange={(e) => setIncludeImages(e.target.checked)} className="checkbox" /><span className="text-text-secondary">{t('include_images')}</span></label>
      <button onClick={onClean} disabled={!selectedPath || cleaning || cleaned} className="btn btn-primary flex items-center gap-2">
        {cleaning ? (
          <><svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>{t('cleaning')}</>
        ) : cleaned ? (
          <><svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg> {t('cleaning_complete')}</>
        ) : (
          <><svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>{t('clean_project')}</>
        )}
      </button>
    </div>
  )
}

function StepPushToGitHub({ cleanedPath, onPush, pushing }: { cleanedPath: string; onPush: (config: RepoConfig, ownerType: string, ownerName: string) => void; pushing: boolean }) {
  const { t } = useLanguage()
  const [showModal, setShowModal] = useState(false)
  const [repoName, setRepoName] = useState('')
  const [description, setDescription] = useState('')
  const [isPrivate, setIsPrivate] = useState(true)
  const [includeImages, setIncludeImages] = useState(true)
  const [createReadme, setCreateReadme] = useState(true)
  const [licenseTemplate, setLicenseTemplate] = useState('mit')
  const [repoType, setRepoType] = useState('standard')
  const [ownerType, setOwnerType] = useState<'user' | 'org'>('user')
  const [organizations, setOrganizations] = useState<Array<{login: string, id: number}>>([])
  const [selectedOrg, setSelectedOrg] = useState('')

  
  useEffect(() => {
    const fetchOrgs = async () => {
      try {
        const orgs = await invoke<Array<{login: string, id: number}>>('get_user_organizations')
        setOrganizations(orgs)
      } catch (err) {
        console.error('Failed to fetch organizations:', err)
      }
    }
    if (showModal) {
      fetchOrgs()
    }
  }, [showModal])

  const handleOpenModal = () => {
    if (cleanedPath) {
      setRepoName(cleanedPath.split(/[/\\]/).pop()?.replace('_LidBridge', '') || '')
      setOwnerType('user')
      setSelectedOrg('')
      setShowModal(true)
    }
  }

  const handlePush = () => {
    const ownerName = ownerType === 'user' ? '' : selectedOrg
    onPush({ name: repoName, description, is_private: isPrivate, include_images: includeImages, create_readme: createReadme, license_template: licenseTemplate, repo_type: repoType }, ownerType, ownerName)
    setShowModal(false)
  }

  return (
    <div className="card mb-6">
      <h3 className="text-lg font-semibold text-text-primary mb-4">{t('step3')}</h3>
      <button onClick={handleOpenModal} disabled={!cleanedPath || pushing} className="btn btn-primary flex items-center gap-2">
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>
        {t('push_github')}
      </button>
      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-bg-secondary border border-border-subtle rounded-lg p-6 w-full max-w-md">
            <h3 className="text-xl font-semibold text-text-primary mb-6">{t('create_repo')}</h3>
        <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2">
              
              <div>
                <label className="block text-text-secondary text-sm mb-2">Push to</label>
                <div className="flex gap-4">
                  <button 
                    onClick={() => setOwnerType('user')} 
                    className={`flex-1 py-2 rounded-md border ${ownerType === 'user' ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'}`}
                  >
                    My Account
                  </button>
                  <button 
                    onClick={() => setOwnerType('org')} 
                    disabled={organizations.length === 0}
                    className={`flex-1 py-2 rounded-md border ${ownerType === 'org' ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'} ${organizations.length === 0 ? 'opacity-50 cursor-not-allowed' : ''}`}
                  >
                    Organization
                  </button>
                </div>
              </div>

              
              {ownerType === 'org' && organizations.length > 0 && (
                <div>
                  <label className="block text-text-secondary text-sm mb-2">Select Organization</label>
                  <select 
                    value={selectedOrg} 
                    onChange={(e) => setSelectedOrg(e.target.value)}
                    className="input w-full"
                  >
                    <option value="">Select organization</option>
                    {organizations.map(org => (
                      <option key={org.id} value={org.login}>{org.login}</option>
                    ))}
                  </select>
                </div>
              )}

              
              <div><label className="block text-text-secondary text-sm mb-2">{t('repo_name')}</label><input type="text" value={repoName} onChange={(e) => setRepoName(e.target.value)} className="input w-full" placeholder="my-project" /></div>
              <div><label className="block text-text-secondary text-sm mb-2">{t('description')}</label><textarea value={description} onChange={(e) => setDescription(e.target.value)} className="input w-full h-24 resize-none" placeholder="A brief description..." /></div>
              <div><label className="block text-text-secondary text-sm mb-2">License</label><select value={licenseTemplate} onChange={(e) => setLicenseTemplate(e.target.value)} className="input w-full"><option value="mit">MIT</option><option value="apache-2.0">Apache-2.0</option><option value="gpl-3.0">GPL-3.0</option><option value="none">None</option></select></div>
              <div><label className="block text-text-secondary text-sm mb-2">Repository Type</label><select value={repoType} onChange={(e) => setRepoType(e.target.value)} className="input w-full"><option value="standard">Standard</option><option value="template">Template</option></select></div>
              <div><label className="block text-text-secondary text-sm mb-2">{t('visibility')}</label><div className="flex gap-4"><button onClick={() => setIsPrivate(false)} className={`flex-1 py-2 rounded-md border ${!isPrivate ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'}`}>{t('public')}</button><button onClick={() => setIsPrivate(true)} className={`flex-1 py-2 rounded-md border ${isPrivate ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'}`}>{t('private')}</button></div></div>
              <div className="space-y-2"><label className="flex items-center gap-3 cursor-pointer"><input type="checkbox" checked={includeImages} onChange={(e) => setIncludeImages(e.target.checked)} className="checkbox" /><span className="text-text-secondary">{t('include_images')}</span></label><label className="flex items-center gap-3 cursor-pointer"><input type="checkbox" checked={createReadme} onChange={(e) => setCreateReadme(e.target.checked)} className="checkbox" /><span className="text-text-secondary">{t('create_readme')}</span></label></div>
            </div>
            <div className="flex gap-3 mt-6"><button onClick={() => setShowModal(false)} className="btn btn-secondary flex-1">{t('cancel')}</button><button onClick={handlePush} disabled={!repoName || (ownerType === 'org' && !selectedOrg)} className="btn btn-primary flex-1">{t('create_push')}</button></div>
          </div>
        </div>
      )}
    </div>
  )
}

function ProgressBar({ progress, runLog, t }: { progress: ProgressState; runLog?: string[]; t?: (key: string) => string }) {
  const { t: contextT } = useLanguage()
  const translate = t || contextT
  return (
    <div className="card space-y-3">
      <div className="flex justify-between text-text-secondary text-sm mb-2">
        <span>{progress.message || translate('scanning')}</span>
        <span>{progress.percentage}%</span>
      </div>
      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${progress.percentage}%` }}></div>
      </div>
      {runLog && runLog.length > 0 && (
        <div className="rounded-lg border border-border-subtle bg-bg-primary/70 p-3 text-sm text-text-secondary">
          <p className="mb-2 font-medium text-text-primary">Run log</p>
          <div className="space-y-1">
            {runLog.slice(-4).reverse().map((item, index) => <p key={`${item}-${index}`} className="truncate">• {item}</p>)}
          </div>
        </div>
      )}
    </div>
  )
}

function Toast({ message, type, onClose }: { message: string; type: 'success' | 'error' | 'warning'; onClose: () => void }) {
  useEffect(() => { 
    const timer = setTimeout(onClose, 8000);
    return () => clearTimeout(timer) 
  }, [onClose])

  const colors = { success: 'border-success', error: 'border-error', warning: 'border-warning' }
  return (<div className={`fixed bottom-4 right-4 bg-bg-tertiary border-l-4 ${colors[type]} rounded-md p-4 shadow-lg z-50`}><p className="text-text-primary">{message}</p></div>)
}

function AIAnalysisModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const { t } = useLanguage()
  if (!isOpen) return null
  return (<div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onClick={onClose}><div className="bg-gradient-to-br from-bg-secondary to-bg-tertiary border border-accent-primary/30 rounded-2xl p-8 w-full max-w-lg" onClick={(e) => e.stopPropagation()}><div className="text-center mb-6"><div className="w-20 h-20 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-2xl flex items-center justify-center mx-auto mb-4"><svg className="w-10 h-10 text-black" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" /></svg></div><h2 className="text-2xl font-bold text-text-primary">{t('ai_analysis_title')}</h2><p className="text-accent-primary text-sm mt-1">{t('ai_analysis_title_sub')}</p></div><div className="space-y-4 mb-6"><div className="bg-bg-primary/50 rounded-xl p-4 border"><div className="flex items-start gap-3"><div className="w-8 h-8 bg-accent-primary/20 rounded-lg flex items-center justify-center"><svg className="w-4 h-4 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg></div><div><h4 className="text-text-primary font-medium">Smart README Generation</h4><p className="text-text-secondary text-sm mt-1">AI generates detailed documentation and file structure analysis</p></div></div></div><div className="bg-bg-primary/50 rounded-xl p-4 border"><div className="flex items-start gap-3"><div className="w-8 h-8 bg-accent-secondary/20 rounded-lg flex items-center justify-center"><svg className="w-4 h-4 text-accent-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg></div><div><h4 className="text-text-primary font-medium">Security Scan</h4><p className="text-text-secondary text-sm mt-1">Detects exposed API keys or secrets before pushing to GitHub</p></div></div></div></div><div className="rounded-xl border border-accent-primary/30 bg-accent-primary/5 p-5"><div className="flex items-start gap-3"><span className="text-lg flex-shrink-0"></span><div><p className="text-text-primary text-sm font-medium">{t('ai_analysis_pro')}</p><p className="text-text-secondary text-sm mt-2">As an independent developer, I cannot provide this feature for free. GitHub Copilot API costs are high, and this helps me maintain the app for everyone.</p></div></div></div><button onClick={onClose} className="btn btn-secondary w-full mt-6">Close</button></div></div>)
}

function HistoryModal({ isOpen, repos, onClose }: { isOpen: boolean; repos: Array<{repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string;}>; onClose: () => void }) {
  const { t } = useLanguage()
  if (!isOpen) return null
  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-bg-secondary border border-border-subtle rounded-2xl p-6 w-full max-w-3xl" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-2xl font-bold text-text-primary">{t('history_title')}</h2>
            <p className="text-text-secondary text-sm">{t('your_repositories')}</p>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text-primary">Close</button>
        </div>
        <div className="space-y-4">
          {repos.length === 0 ? (
            <div className="p-6 bg-bg-primary rounded-xl text-text-secondary text-sm">No repository history found.</div>
          ) : repos.map((repo, idx) => (
            <a key={idx} href={repo.repo_url} target="_blank" rel="noopener noreferrer" className="block p-4 border border-border-subtle rounded-xl hover:border-accent-primary transition">
              <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
                <div>
                  <p className="text-text-primary font-semibold">{repo.repo_name}</p>
                  <p className="text-text-secondary text-sm">{t('owner')}: {repo.owner_type === 'org' ? repo.owner_name : repo.owner_name}</p>
                </div>
                <div className="text-text-muted text-sm">{repo.created_at}</div>
              </div>
            </a>
          ))}
        </div>
      </div>
    </div>
  )
}

function AboutModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const { t } = useLanguage()
  if (!isOpen) return null
  const links = [
    { label: t('parent_company'), url: 'https://lidprex.onrender.com/', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
    { label: t('the_lab'), url: 'https://lidprex-labs.onrender.com/', icon: 'M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z' },
    { label: t('lead_developer'), url: 'http://github.com/bxat01', icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4' },
    { label: t('organization'), url: 'http://github.com/lidprex', icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z' },
  ]
  return (<div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onClick={onClose}><div className="bg-gradient-to-br from-bg-secondary to-bg-tertiary border border-accent-secondary/30 rounded-2xl p-8 w-full max-w-md" onClick={(e) => e.stopPropagation()}><div className="text-center mb-6"><img src="https://res.cloudinary.com/ddqedxovk/image/upload/v1777644756/zdmst5ng01o20lam01ou.png" alt="Logo" className="w-24 h-24 mx-auto rounded-2xl mb-4" /><h2 className="text-2xl font-bold text-text-primary">{t('about_title')}</h2><p className="text-text-secondary text-sm mt-1">v1.0.0</p></div><div className="text-center mb-6"><p className="text-text-muted text-sm">{t('created_by')}</p><p className="text-xl font-bold bg-gradient-to-r from-accent-primary to-accent-secondary bg-clip-text text-transparent">Lidprex Labs</p></div><div className="space-y-3">{links.map((link, i) => (<a key={i} href={link.url} target="_blank" rel="noopener noreferrer" className="flex items-center gap-3 p-3 bg-bg-primary/50 rounded-xl border hover:border-accent-secondary/50"><div className="w-8 h-8 bg-accent-secondary/20 rounded-lg flex items-center justify-center"><svg className="w-4 h-4 text-accent-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={link.icon} /></svg></div><span className="text-text-secondary">{link.label}</span><svg className="w-4 h-4 text-text-muted ml-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg></a>))}</div><button onClick={onClose} className="btn btn-secondary w-full mt-6">Close</button></div></div>)
}

function Dashboard() {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)
  const [selectedPath, setSelectedPath] = useState('')
  const [cleanedPath, setCleanedPath] = useState('')
  const [scanning, setScanning] = useState(false)
  const [cleaning, setCleaning] = useState(false)
  const [cleaned, setCleaned] = useState(false)
  const [pushing, setPushing] = useState(false)
  const [progress, setProgress] = useState<ProgressState>({ percentage: 0, message: '' })
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'warning' } | null>(null)
  const [showAIModal, setShowAIModal] = useState(false)
  const [showAboutModal, setShowAboutModal] = useState(false)
  const [showHistory, setShowHistory] = useState(false)
  const [repoHistory, setRepoHistory] = useState<Array<{repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string;}>>([])
  const [scanResult, setScanResult] = useState<ScanResult | null>(null)
  const [runLog, setRunLog] = useState<string[]>([])
  const [targetPath, setTargetPath] = useState('')
  const [includeImages, setIncludeImages] = useState(false)
  const [secretReplacements, setSecretReplacements] = useState<Record<string, string>>({})
  const { t, isRTL } = useLanguage()

  useEffect(() => { checkSession() }, [])
  
  useEffect(() => {
    const unlisten = listen<CleanProgress>('cleaning-progress', (event) => {
      const p = event.payload
      let message = ''
      switch (p.phase) {
        case 'scanning': message = t('scanning'); break
        case 'copying': message = t('copying'); break
        case 'cleaning': message = t('cleaning'); break
        case 'complete': message = t('cleaning_complete'); break
        default: message = p.phase
      }
      setProgress({ percentage: p.percentage, message })
    })
    return () => { unlisten.then(fn => fn()) }
  }, [t])

  const appendRunLog = (message: string) => {
    const stamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    setRunLog((prev) => [...prev.slice(-19), `${stamp} • ${message}`])
  }

  const checkSession = async () => {
    try {
      const session = await invoke<User | null>('get_session')
      setUser(session)
    } catch (err) {
      console.error(err)
    } finally {
      setLoading(false)
    }
  }
  
  const handleLogin = async () => {
    try {
      setToast({ message: 'Opening GitHub in browser...', type: 'warning' })
      await invoke<string>('start_oauth')
      setToast({ message: 'Authorize in your browser. You will be signed in automatically.', type: 'success' })
    } catch (err) {
      setToast({ message: `Failed to start OAuth: ${err}`, type: 'error' })
    }
  }
  
  const handleLogout = async () => {
    try {
      await invoke('logout')
      setUser(null)
      setSelectedPath('')
      setCleanedPath('')
      setScanResult(null)
      setCleaned(false)
    } catch (err) {
      setToast({ message: 'Failed to logout', type: 'error' })
    }
  }
  
  const fetchRepoHistory = async () => {
    try {
      const history = await invoke<Array<{repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string;}>>('get_repo_history')
      setRepoHistory(history)
    } catch (err) {
      console.error('Failed to load repo history:', err)
    }
  }

  const handleSelectFolder = async () => {
  try {
    const folder = await openDialog({
      directory: true,
      title: 'Select Project Folder'
    })
    if (folder && typeof folder === 'string') {
      setSelectedPath(folder)
      setCleanedPath('')
      setCleaned(false)
      setScanResult(null)
      setProgress({ percentage: 0, message: t('scanning') })
      
      const projectName = folder.split(/[/\\]/).pop() || 'project'
      const parentPath = folder.substring(0, folder.lastIndexOf(folder.includes('/') ? '/' : '\\'))
      const pathSeparator = parentPath.includes('\\') ? '\\' : '/'
      const autoTargetPath = `${parentPath}${parentPath.endsWith(pathSeparator) ? '' : pathSeparator}${projectName}_LidBridge`
      setTargetPath(autoTargetPath)
      appendRunLog(`Scanning project ${projectName}`)
      
      const result = await invoke<ScanResult>('scan_project_command', { 
        sourceDir: folder, 
        includeImages: false 
      })
      setScanResult(result)
      appendRunLog(`Scan completed: ${result.total_files} files, ${result.secrets_count} potential secrets`)
      setSecretReplacements(Object.fromEntries((result.secret_matches || []).map((match) => [match, ''])))
      setToast({ 
        message: `Found ${result.total_files} files (${result.clean_files} clean)`, 
        type: result.secrets_count > 0 ? 'warning' : 'success' 
      })
    }
  } catch (err) {
    console.error("Error selecting folder:", err)
    setToast({ message: 'Failed to select folder', type: 'error' })
  }
}

  
  const handleSelectTargetFolder = async () => {
  try {
    const folder = await openDialog({ directory: true, title: 'Select Destination Folder' })
    if (folder && typeof folder === 'string') {
      setTargetPath(folder)
    }
  } catch (err) {
    setToast({ message: 'Failed to select destination folder', type: 'error' })
  }
}

  
  const handleClean = async () => {
  if (!selectedPath) return
  
  setCleaning(true)
  setProgress({ percentage: 0, message: t('scanning') })
  if (scanResult && scanResult.secrets_count > 0) {
    setToast({ message: `Potential secrets detected (${scanResult.secrets_count}). Review them before publishing.`, type: 'warning' })
  }
  try {
    const projectName = selectedPath.split(/[/\\]/).pop() || 'project'
    const parentPath = selectedPath.substring(0, selectedPath.lastIndexOf(selectedPath.includes('/') ? '/' : '\\'))
    const baseOutputDir = targetPath || parentPath
    const pathSeparator = baseOutputDir.includes('\\') ? '\\' : '/'
    const outputDir = `${baseOutputDir}${baseOutputDir.endsWith(pathSeparator) ? '' : pathSeparator}${projectName}_LidBridge`

      
      const result = await invoke<CleanResult>('start_cleaning_command', {
        sourceDir: selectedPath,
        outputDir,
        options: {
          mode: 'clean',
          include_images: includeImages,
          include_videos: false,
          include_documents: false,
          create_readme: false,
          secret_replacements: Object.entries(secretReplacements).filter(([, value]) => value.trim()).map(([name, replacement]) => ({ name, replacement })),
        }
      })
      if (result.success) {
  appendRunLog(`Cleaning completed: ${result.copied_files} files copied`)
  setCleanedPath(result.cleaned_path)
  setCleaned(true)

        setProgress({ percentage: 100, message: t('cleaning_complete') })
        setToast({ message: `Cleaned ${result.copied_files} files`, type: 'success' })
        result.warnings.forEach(warning => setToast({ message: warning, type: 'warning' }))
      } else {
        setToast({ message: 'Failed to clean project', type: 'error' })
      }
    } catch (err) {
      setToast({ message: `Error cleaning project: ${err}`, type: 'error' })
    } finally {
      setCleaning(false)
    }
  }
  
  const handlePush = async (config: RepoConfig, ownerType: string, ownerName: string) => {
  if (!cleanedPath) {
    setToast({ message: 'Please clean a project first!', type: 'warning' })
    return
  }
  
  setPushing(true)
  setProgress({ percentage: 0, message: 'Initializing repository...' })
  
  try {
    appendRunLog(`Creating repository ${config.name}`)
    const repoUrl = await invoke<string>('create_and_push_command', { 
      path: cleanedPath, 
      config, 
      ownerType, 
      ownerName 
    })
    appendRunLog(`Repository published: ${repoUrl}`)
    
    setProgress({ percentage: 100, message: t('push_complete') })
    
    
    const userConfirmed = window.confirm(
      `Repository created successfully!\n\nClick OK to open: ${repoUrl}\n\nOr press Cancel to close.`
    );
    
    if (userConfirmed) {
      window.open(repoUrl, '_blank');
    }
    
    setToast({ message: 'Successfully pushed to GitHub!', type: 'success' })
    await fetchRepoHistory()
  } catch (err) {
    const errorMsg = err as string;
    
    
    if (errorMsg.includes("403") || errorMsg.includes("Resource not accessible")) {
      const message = `Permission Error (403 Forbidden)

Your GitHub OAuth token doesn't have the required permissions.

Follow these steps:
1. Go to: https://github.com/settings/developers
2. Click on your LidBridge OAuth App
3. Go to "Permissions" tab
4. Ensure these are enabled:
   ✓ repo (full control of repositories)
   ✓ write:org (write access to organizations)
   ✓ read:user (read user data)
5. Click "Update permissions"
6. Then LOGOUT and LOGIN again in LidBridge`;
      
      const confirmed = window.confirm(message + "\n\nClick OK to open GitHub settings");
      if (confirmed) {
        window.open("https://github.com/settings/developers", '_blank');
      }
      setToast({ message: "Permission error - check GitHub OAuth app settings", type: 'error' });
    } else if (errorMsg.includes("INSTALL_REQUIRED") || errorMsg.includes("403")) {
      
      if (ownerType === 'org') {
        const installUrl = "https://github.com/apps/lidbridge/installations/new";
        const confirmed = window.confirm(
          `GitHub App not installed.\n\nClick OK to install the app, then try again.\n\nCancel to close.`
        );
        if (confirmed) {
          window.open(installUrl, '_blank');
        }
        setToast({ 
          message: "Please install the GitHub App, then try pushing again.", 
          type: 'warning' 
        });
      } else {
        
        setToast({ message: `Failed to push: Check permissions and try logging out/in again`, type: 'error' });
      }
    } else {
      setToast({ message: `Failed to push: ${errorMsg}`, type: 'error' });
    }
  } finally {
    setPushing(false)
  }
};

  if (loading) return (
    <div className="min-h-screen bg-bg-primary flex items-center justify-center">
      <div className="animate-spin w-8 h-8 border-2 border-accent-primary border-t-transparent rounded-full"></div>
    </div>
  )

  return (
    <div className={`min-h-screen bg-bg-primary flex flex-col ${isRTL ? 'rtl' : 'ltr'}`} dir={isRTL ? 'rtl' : 'ltr'}>
      <Header user={user} onLogout={handleLogout} onAIAnalysis={() => setShowAIModal(true)} onAbout={() => setShowAboutModal(true)} onHistory={() => { setShowHistory(true); fetchRepoHistory() }} />
      {!user ? <AuthScreen onLogin={handleLogin} /> : (
        <main className="flex-1 p-8 max-w-3xl mx-auto w-full">
          <StepSelectProject selectedPath={selectedPath} onSelectFolder={handleSelectFolder} />
          <StepCleanProject 
            selectedPath={selectedPath}
            targetPath={targetPath}
            onSelectTargetFolder={handleSelectTargetFolder}
            onClean={handleClean}
            cleaning={cleaning}
            cleaned={cleaned}
            scanResult={scanResult}
            includeImages={includeImages}
            setIncludeImages={setIncludeImages}
          />
          <StepPushToGitHub cleanedPath={cleanedPath} onPush={handlePush} pushing={pushing} />
          {(cleaning || pushing) && <ProgressBar progress={progress} runLog={runLog} t={(key: string) => t(key as TranslationKey)} />}
        </main>
      )}
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
      <AIAnalysisModal isOpen={showAIModal} onClose={() => setShowAIModal(false)} />
      <HistoryModal isOpen={showHistory} repos={repoHistory} onClose={() => setShowHistory(false)} />
      <AboutModal isOpen={showAboutModal} onClose={() => setShowAboutModal(false)} />
    </div>
  )
}

function PageShell() {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)
  const [selectedPath, setSelectedPath] = useState('')
  const [cleanedPath, setCleanedPath] = useState('')
  const [scanning, setScanning] = useState(false)
  const [cleaning, setCleaning] = useState(false)
  const [cleaned, setCleaned] = useState(false)
  const [pushing, setPushing] = useState(false)
  const [pushResultUrl, setPushResultUrl] = useState('')
  const [progress, setProgress] = useState<ProgressState>({ percentage: 0, message: '' })
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'warning' } | null>(null)
  const [showAIModal, setShowAIModal] = useState(false)
  const [showAboutModal, setShowAboutModal] = useState(false)
  const [showHistory, setShowHistory] = useState(false)
  const [repoHistory, setRepoHistory] = useState<Array<{repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string;}>>([])
  const [scanResult, setScanResult] = useState<ScanResult | null>(null)
  const [runLog, setRunLog] = useState<string[]>([])
  const [targetPath, setTargetPath] = useState('')
  const [includeImages, setIncludeImages] = useState(false)
  const [createReadme, setCreateReadme] = useState(false)
  const [secretReplacements, setSecretReplacements] = useState<Record<string, string>>({})
  const { t, isRTL, lang, setLang } = useLanguage()

  useEffect(() => { checkSession() }, [])
  useEffect(() => {
    isPermissionGranted().then(granted => {
      if (!granted) requestPermission()
    }).catch(() => {})
  }, [])

  useEffect(() => {
    const unlisten = listen<CleanProgress>('cleaning-progress', (event) => {
      const p = event.payload
      let message = ''
      switch (p.phase) {
        case 'scanning': message = t('scanning'); break
        case 'copying': message = t('copying'); break
        case 'cleaning': message = t('cleaning'); break
        case 'complete': message = t('cleaning_complete'); break
        default: message = p.phase
      }
      setProgress({ percentage: p.percentage, message })
    })
    return () => { unlisten.then(fn => fn()) }
  }, [t])

  useEffect(() => {
    const unlistenOAuth = listen<string>('oauth-code-received', async (event) => {
      const code = event.payload
      try {
        setToast({ message: 'Completing authentication...', type: 'success' })
        const user = await invoke<User>('complete_oauth', { code })
        if (user) {
          setUser(user)
          fetchRepoHistory()
          setToast({ message: 'Successfully logged in!', type: 'success' })
        } else {
          setToast({ message: 'Authentication completed but session could not be loaded.', type: 'error' })
        }
      } catch (err) {
        console.error('OAuth completion error:', err)
        setToast({ message: `Authentication failed: ${err}`, type: 'error' })
      }
    })
    const unlistenOAuthStatus = listen<string>('oauth-status', (event) => {
      const status = event.payload
      const isError = status.toLowerCase().includes('error') || status.toLowerCase().includes('failed')
      const isScopeWarning = status.toLowerCase().includes('lacks') || status.toLowerCase().includes('scope')
      if (isError) {
        setToast({ message: status, type: 'error' })
      } else if (isScopeWarning) {
        setToast({ message: status, type: 'warning' })
      }
    })
    return () => { unlistenOAuth.then(fn => fn()); unlistenOAuthStatus.then(fn => fn()) }
  }, [])

  const appendRunLog = (message: string) => {
    const stamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    setRunLog((prev) => [...prev.slice(-19), `${stamp} • ${message}`])
  }

  const fetchRepoHistory = async () => {
    try {
      const history = await invoke<Array<{repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string;}>>('get_repo_history')
      setRepoHistory(history)
    } catch (err) {
      console.error('Failed to load repo history:', err)
    }
  }

  const checkSession = async () => {
    try {
      const session = await invoke<User | null>('get_session')
      setUser(session)
      if (session) fetchRepoHistory()
    } catch (err) {
      console.error(err)
    } finally {
      setLoading(false)
    }
  }

  const handleLogin = async () => {
    try {
      setToast({ message: 'Opening GitHub login...', type: 'warning' })
      await invoke('start_oauth')
    } catch (err) {
      setToast({ message: 'Failed to start OAuth', type: 'error' })
    }
  }

  const handleLogout = async () => {
    try {
      await invoke('logout')
      setUser(null)
      setSelectedPath('')
      setCleanedPath('')
      setScanResult(null)
      setCleaned(false)
    } catch (err) {
      setToast({ message: 'Failed to logout', type: 'error' })
    }
  }

  const handlePersonalTokenLogin = async (token: string) => {
    try {
      await invoke('save_github_token', { token })
      const session = await invoke<User | null>('get_session')
      if (!session) throw new Error('GitHub did not return a session')
      setUser(session)
      setToast({ message: 'Successfully logged in with a personal token.', type: 'success' })
    } catch (err) {
      setToast({ message: `Personal token login failed: ${err}`, type: 'error' })
      throw err
    }
  }

  const handleSelectFolder = async () => {
    try {
      const folder = await openDialog({ directory: true, title: 'Select Project Folder' })
      if (folder && typeof folder === 'string') {
        setSelectedPath(folder)
        setCleanedPath('')
        setCleaned(false)
        setScanResult(null)
        setScanning(true)
        setProgress({ percentage: 0, message: t('scanning') })
        const projectName = folder.split(/[/\\]/).pop() || 'project'
        appendRunLog(`Scanning project ${projectName}`)
        const parentPath = folder.substring(0, folder.lastIndexOf(folder.includes('/') ? '/' : '\\'))
        const pathSeparator = parentPath.includes('\\') ? '\\' : '/'
        const autoTargetPath = `${parentPath}${parentPath.endsWith(pathSeparator) ? '' : pathSeparator}${projectName}_LidBridge`
        setTargetPath(autoTargetPath)
        const result = await invoke<ScanResult>('scan_project_command', { sourceDir: folder, includeImages: false })
        setScanResult(result)
        appendRunLog(`Scan completed: ${result.total_files} files, ${result.secrets_count} potential secrets`)
        setSecretReplacements(Object.fromEntries((result.secret_matches || []).map((match) => [match, ''])))
        setToast({ message: `Found ${result.total_files} files (${result.clean_files} clean)`, type: result.secrets_count > 0 ? 'warning' : 'success' })
      }
    } catch (err) {
      console.error('Error selecting folder:', err)
      setToast({ message: 'Failed to select folder', type: 'error' })
    } finally {
      setScanning(false)
    }
  }

  const handleSelectTargetFolder = async () => {
    try {
      const folder = await openDialog({ directory: true, title: 'Select Destination Folder' })
      if (folder && typeof folder === 'string') {
        const projectName = selectedPath.split(/[/\\]/).pop() || 'project'
        const separator = folder.includes('\\') ? '\\' : '/'
        setTargetPath(`${folder}${folder.endsWith(separator) ? '' : separator}${projectName}_LidBridge`)
      }
    } catch (err) {
      setToast({ message: 'Failed to select destination folder', type: 'error' })
    }
  }

  const handleSecretReplacementChange = (secretName: string, value: string) => {
    setSecretReplacements((prev) => ({ ...prev, [secretName]: value }))
  }

  const handleApplySecretReplacements = () => {
    setToast({ message: 'Secret replacements prepared for the cleaned output.', type: 'success' })
  }

  const handleClean = async () => {
    if (!selectedPath) return
    setCleaning(true)
    setProgress({ percentage: 0, message: t('scanning') })
    if (scanResult && scanResult.secrets_count > 0) {
      setToast({ message: `Potential secrets detected (${scanResult.secrets_count}). Review them before publishing.`, type: 'warning' })
    }
    try {
      const projectName = selectedPath.split(/[/\\]/).pop() || 'project'
      const parentPath = selectedPath.substring(0, selectedPath.lastIndexOf(selectedPath.includes('/') ? '/' : '\\'))
      const outputDir = targetPath || `${parentPath}${parentPath.includes('\\') ? '\\' : '/'}${projectName}_LidBridge`
      appendRunLog(`Cleaning project into ${outputDir}`)
      const result = await invoke<CleanResult>('start_cleaning_command', {
        sourceDir: selectedPath,
        outputDir,
        options: {
          mode: 'clean',
          include_images: includeImages,
          include_videos: false,
          include_documents: false,
          create_readme: createReadme,
          secret_replacements: Object.entries(secretReplacements).filter(([, value]) => value.trim()).map(([name, replacement]) => ({ name, replacement })),
        }
      })
      if (result.success) {
        appendRunLog(`Cleaning completed: ${result.copied_files} files copied`)
        setCleanedPath(result.cleaned_path)
        setCleaned(true)
        setProgress({ percentage: 100, message: t('cleaning_complete') })
        setToast({ message: `Cleaned ${result.copied_files} files`, type: 'success' })
        result.warnings.forEach((warning) => setToast({ message: warning, type: 'warning' }))
      } else {
        setToast({ message: 'Failed to clean project', type: 'error' })
      }
    } catch (err) {
      setToast({ message: `Error cleaning project: ${err}`, type: 'error' })
    } finally {
      setCleaning(false)
    }
  }

  const handlePush = async (config: RepoConfig, ownerType: string, ownerName: string) => {
    if (!cleanedPath) {
      setToast({ message: 'Please clean a project first!', type: 'warning' })
      return
    }
    setPushing(true)
    setPushResultUrl('')
    setProgress({ percentage: 0, message: 'Creating repository...' })
    try {
      appendRunLog(`Creating repository ${config.name}`)
      const repoUrl = await invoke<string>('create_and_push_command', { path: cleanedPath, config, ownerType, ownerName })
      appendRunLog(`Repository published: ${repoUrl}`)
      setProgress({ percentage: 100, message: t('push_complete') })
      setPushResultUrl(repoUrl)
      setToast({ message: 'Successfully pushed to GitHub!', type: 'success' })
      await fetchRepoHistory()
      isPermissionGranted().then(granted => {
        if (granted) sendNotification({ title: 'LidBridge', body: `Repository "${config.name}" pushed successfully!` })
      }).catch(() => {})
    } catch (err) {
      const errorMsg = err as string
      if (errorMsg.includes('403') || errorMsg.includes('Resource not accessible')) {
        const message = `Permission Error (403 Forbidden)\n\nYour GitHub token doesn't have the required permissions.\n\nFollow these steps:\n1. Go to GitHub settings\n2. Ensure the token has repo access\n3. Try again.`
        const confirmed = window.confirm(message + '\n\nClick OK to open GitHub settings')
        if (confirmed) window.open('https://github.com/settings/tokens/new', '_blank')
        setToast({ message: 'Permission error - check GitHub token permissions', type: 'error' })
      } else {
        setToast({ message: `Failed to push: ${errorMsg}`, type: 'error' })
      }
    } finally {
      setPushing(false)
    }
  }

  const handleResetAfterPush = () => {
    setPushResultUrl('')
    setSelectedPath('')
    setCleanedPath('')
    setCleaned(false)
    setScanResult(null)
    setTargetPath('')
    setProgress({ percentage: 0, message: '' })
  }

  return (
    <DashboardUI
      user={user}
      loading={loading}
      selectedPath={selectedPath}
      cleanedPath={cleanedPath}
      scanning={scanning}
      cleaning={cleaning}
      cleaned={cleaned}
      pushing={pushing}
      progress={progress}
      toast={toast}
      repoHistory={repoHistory}
      scanResult={scanResult}
      runLog={runLog}
      targetPath={targetPath}
      includeImages={includeImages}
      createReadme={createReadme}
      showAIModal={showAIModal}
      showAboutModal={showAboutModal}
      showHistory={showHistory}
      isRTL={isRTL}
      t={(key: string) => t(key as TranslationKey)}
      setIncludeImages={setIncludeImages}
      setCreateReadme={setCreateReadme}
      onLogin={handleLogin}
      onPersonalTokenLogin={handlePersonalTokenLogin}
      onLogout={handleLogout}
      onAIAnalysis={() => setShowAIModal(true)}
      onAbout={() => setShowAboutModal(true)}
      onHistory={() => { setShowHistory(true); fetchRepoHistory() }}
      onSelectFolder={handleSelectFolder}
      onSelectTargetFolder={handleSelectTargetFolder}
      onClean={handleClean}
      onPush={handlePush}
      pushResultUrl={pushResultUrl}
      onResetAfterPush={handleResetAfterPush}
      onCloseToast={() => setToast(null)}
      onCloseAIModal={() => setShowAIModal(false)}
      onCloseAboutModal={() => setShowAboutModal(false)}
      onCloseHistoryModal={() => setShowHistory(false)}
      onOpenSecretReview={() => {}}
      secretReplacements={secretReplacements}
      onSecretReplacementChange={handleSecretReplacementChange}
      onApplySecretReplacements={handleApplySecretReplacements}
      lang={lang}
      setLang={setLang}
    />
  )
}

export default function Home() {
  return (<LanguageProvider><PageShell /></LanguageProvider>)
}
