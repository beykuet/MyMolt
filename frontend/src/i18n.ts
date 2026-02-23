// Simple i18n system for MyMolt frontend
// Translations for EN, DE, FR, ES, IT

export type Locale = 'en' | 'de' | 'fr' | 'es' | 'it';

const translations: Record<Locale, Record<string, string>> = {
    en: {
        // Lobby
        'lobby.title': 'Who is using MyMolt?',
        'lobby.subtitle': 'Select your profile to enter your sovereign space.',
        'lobby.root.title': 'Root',
        'lobby.root.desc': 'System Admin & Security',
        'lobby.adult.title': 'Adult',
        'lobby.adult.desc': 'Productivity & Finance',
        'lobby.child.title': 'Child',
        'lobby.child.desc': 'Safe Learning & Play',
        'lobby.senior.title': 'Senior',
        'lobby.senior.desc': 'Simplified & Voice First',
        'lobby.enter_token': 'Enter your pairing token:',

        // Dashboard
        'dashboard.greeting': 'Welcome back',
        'dashboard.logout': 'Logout',
        'dashboard.chat.placeholder': 'Ask MyMolt anything...',
        'dashboard.chat.send': 'Send',

        // Widgets
        'widget.diary.title': 'Diary',
        'widget.diary.empty': 'No entries yet. Start writing!',
        'widget.diary.placeholder': 'Write a diary entry...',
        'widget.diary.save': 'Save Entry',
        'widget.vpn.title': 'VPN Peers',
        'widget.vpn.add': 'Add Peer',
        'widget.vpn.delete': 'Delete',
        'widget.vpn.empty': 'No VPN peers configured.',
        'widget.vault.title': 'Encrypted Vault',
        'widget.vault.empty': 'Vault is empty.',
        'widget.status.title': 'System Health',
        'widget.panic.title': 'Emergency Stop',

        // Settings
        'settings.model': 'AI Model',
        'settings.pairing': 'Pairing',
        'settings.voice_echo': 'Voice Echo',

        // Errors
        'error.unauthorized': 'Unauthorized — please log in again.',
        'error.forbidden': 'You do not have permission for this action.',
        'error.rate_limit': 'Too many requests. Please wait.',
        'error.generic': 'Something went wrong.',
    },

    de: {
        'lobby.title': 'Wer nutzt MyMolt?',
        'lobby.subtitle': 'Wähle dein Profil, um deinen souveränen Raum zu betreten.',
        'lobby.root.title': 'Root',
        'lobby.root.desc': 'Systemadmin & Sicherheit',
        'lobby.adult.title': 'Erwachsener',
        'lobby.adult.desc': 'Produktivität & Finanzen',
        'lobby.child.title': 'Kind',
        'lobby.child.desc': 'Sicheres Lernen & Spielen',
        'lobby.senior.title': 'Senior',
        'lobby.senior.desc': 'Vereinfacht & Sprachgesteuert',
        'lobby.enter_token': 'Pairing-Token eingeben:',
        'dashboard.greeting': 'Willkommen zurück',
        'dashboard.logout': 'Abmelden',
        'dashboard.chat.placeholder': 'Frag MyMolt etwas...',
        'dashboard.chat.send': 'Senden',
        'widget.diary.title': 'Tagebuch',
        'widget.diary.empty': 'Noch keine Einträge. Fang an zu schreiben!',
        'widget.diary.placeholder': 'Tagebucheintrag schreiben...',
        'widget.diary.save': 'Speichern',
        'widget.vpn.title': 'VPN-Peers',
        'widget.vpn.add': 'Peer hinzufügen',
        'widget.vpn.delete': 'Löschen',
        'widget.vpn.empty': 'Keine VPN-Peers konfiguriert.',
        'widget.vault.title': 'Verschlüsselter Tresor',
        'widget.vault.empty': 'Tresor ist leer.',
        'widget.status.title': 'Systemzustand',
        'widget.panic.title': 'Notaus',
        'settings.model': 'KI-Modell',
        'settings.pairing': 'Kopplung',
        'settings.voice_echo': 'Sprach-Echo',
        'error.unauthorized': 'Nicht autorisiert — bitte erneut anmelden.',
        'error.forbidden': 'Keine Berechtigung für diese Aktion.',
        'error.rate_limit': 'Zu viele Anfragen. Bitte warten.',
        'error.generic': 'Etwas ist schiefgelaufen.',
    },

    fr: {
        'lobby.title': 'Qui utilise MyMolt ?',
        'lobby.subtitle': 'Sélectionnez votre profil pour entrer dans votre espace souverain.',
        'lobby.root.title': 'Root',
        'lobby.root.desc': 'Admin Système & Sécurité',
        'lobby.adult.title': 'Adulte',
        'lobby.adult.desc': 'Productivité & Finance',
        'lobby.child.title': 'Enfant',
        'lobby.child.desc': 'Apprentissage & Jeu Sécurisé',
        'lobby.senior.title': 'Senior',
        'lobby.senior.desc': 'Simplifié & Vocal',
        'lobby.enter_token': 'Entrez votre jeton de couplage :',
        'dashboard.greeting': 'Content de vous revoir',
        'dashboard.logout': 'Déconnexion',
        'dashboard.chat.placeholder': 'Demandez à MyMolt...',
        'dashboard.chat.send': 'Envoyer',
        'widget.diary.title': 'Journal',
        'widget.diary.empty': 'Pas encore d\'entrées. Commencez à écrire !',
        'widget.diary.placeholder': 'Écrire une entrée...',
        'widget.diary.save': 'Enregistrer',
        'widget.vpn.title': 'Pairs VPN',
        'widget.vpn.add': 'Ajouter un pair',
        'widget.vpn.delete': 'Supprimer',
        'widget.vpn.empty': 'Aucun pair VPN configuré.',
        'widget.vault.title': 'Coffre-fort Chiffré',
        'widget.vault.empty': 'Le coffre-fort est vide.',
        'widget.status.title': 'Santé du Système',
        'widget.panic.title': 'Arrêt d\'Urgence',
        'settings.model': 'Modèle IA',
        'settings.pairing': 'Couplage',
        'settings.voice_echo': 'Écho Vocal',
        'error.unauthorized': 'Non autorisé — veuillez vous reconnecter.',
        'error.forbidden': 'Vous n\'avez pas la permission pour cette action.',
        'error.rate_limit': 'Trop de requêtes. Veuillez patienter.',
        'error.generic': 'Une erreur est survenue.',
    },

    es: {
        'lobby.title': '¿Quién usa MyMolt?',
        'lobby.subtitle': 'Selecciona tu perfil para entrar en tu espacio soberano.',
        'lobby.root.title': 'Root',
        'lobby.root.desc': 'Admin del Sistema & Seguridad',
        'lobby.adult.title': 'Adulto',
        'lobby.adult.desc': 'Productividad & Finanzas',
        'lobby.child.title': 'Niño',
        'lobby.child.desc': 'Aprendizaje Seguro & Juego',
        'lobby.senior.title': 'Senior',
        'lobby.senior.desc': 'Simplificado & Voz Primero',
        'lobby.enter_token': 'Ingresa tu token de emparejamiento:',
        'dashboard.greeting': 'Bienvenido de nuevo',
        'dashboard.logout': 'Cerrar sesión',
        'dashboard.chat.placeholder': 'Pregúntale a MyMolt...',
        'dashboard.chat.send': 'Enviar',
        'widget.diary.title': 'Diario',
        'widget.diary.empty': 'Sin entradas. ¡Empieza a escribir!',
        'widget.diary.placeholder': 'Escribe una entrada...',
        'widget.diary.save': 'Guardar',
        'widget.vpn.title': 'Peers VPN',
        'widget.vpn.add': 'Agregar Peer',
        'widget.vpn.delete': 'Eliminar',
        'widget.vpn.empty': 'Ningún peer VPN configurado.',
        'widget.vault.title': 'Bóveda Cifrada',
        'widget.vault.empty': 'La bóveda está vacía.',
        'widget.status.title': 'Salud del Sistema',
        'widget.panic.title': 'Parada de Emergencia',
        'settings.model': 'Modelo IA',
        'settings.pairing': 'Emparejamiento',
        'settings.voice_echo': 'Eco de Voz',
        'error.unauthorized': 'No autorizado — inicia sesión nuevamente.',
        'error.forbidden': 'No tienes permiso para esta acción.',
        'error.rate_limit': 'Demasiadas solicitudes. Por favor espera.',
        'error.generic': 'Algo salió mal.',
    },

    it: {
        'lobby.title': 'Chi usa MyMolt?',
        'lobby.subtitle': 'Seleziona il tuo profilo per entrare nel tuo spazio sovrano.',
        'lobby.root.title': 'Root',
        'lobby.root.desc': 'Admin Sistema & Sicurezza',
        'lobby.adult.title': 'Adulto',
        'lobby.adult.desc': 'Produttività & Finanza',
        'lobby.child.title': 'Bambino',
        'lobby.child.desc': 'Apprendimento Sicuro & Gioco',
        'lobby.senior.title': 'Senior',
        'lobby.senior.desc': 'Semplificato & Voce Prima',
        'lobby.enter_token': 'Inserisci il tuo token di accoppiamento:',
        'dashboard.greeting': 'Bentornato',
        'dashboard.logout': 'Disconnetti',
        'dashboard.chat.placeholder': 'Chiedi a MyMolt...',
        'dashboard.chat.send': 'Invia',
        'widget.diary.title': 'Diario',
        'widget.diary.empty': 'Nessuna voce. Inizia a scrivere!',
        'widget.diary.placeholder': 'Scrivi una voce...',
        'widget.diary.save': 'Salva',
        'widget.vpn.title': 'Peer VPN',
        'widget.vpn.add': 'Aggiungi Peer',
        'widget.vpn.delete': 'Elimina',
        'widget.vpn.empty': 'Nessun peer VPN configurato.',
        'widget.vault.title': 'Cassaforte Crittografata',
        'widget.vault.empty': 'La cassaforte è vuota.',
        'widget.status.title': 'Stato del Sistema',
        'widget.panic.title': 'Arresto di Emergenza',
        'settings.model': 'Modello IA',
        'settings.pairing': 'Accoppiamento',
        'settings.voice_echo': 'Eco Vocale',
        'error.unauthorized': 'Non autorizzato — effettua nuovamente l\'accesso.',
        'error.forbidden': 'Non hai i permessi per questa azione.',
        'error.rate_limit': 'Troppe richieste. Attendere prego.',
        'error.generic': 'Qualcosa è andato storto.',
    },
};

let currentLocale: Locale = (
    (typeof navigator !== 'undefined' && navigator.language?.slice(0, 2)) || 'en'
) as Locale;

// Fallback to 'en' if detected locale isn't supported
if (!translations[currentLocale]) {
    currentLocale = 'en';
}

export function setLocale(locale: Locale) {
    currentLocale = locale;
    localStorage.setItem('mymolt_locale', locale);
}

export function getLocale(): Locale {
    return currentLocale;
}

export function t(key: string): string {
    return translations[currentLocale]?.[key] ?? translations.en[key] ?? key;
}

// Initialize from localStorage if available
const storedLocale = typeof localStorage !== 'undefined' && localStorage.getItem('mymolt_locale');
if (storedLocale && translations[storedLocale as Locale]) {
    currentLocale = storedLocale as Locale;
}

export default { t, setLocale, getLocale };
