// @generated from gherkin-languages.json — do not edit by hand
use super::Dialect;

impl Dialect {
    pub const EN: Dialect = Dialect {
        code: "en",
        name: "English",
        native: "English",
        feature: &["Feature", "Business Need", "Ability"],
        rule: &["Rule"],
        background: &["Background"],
        scenario: &["Example", "Scenario"],
        scenario_outline: &["Scenario Outline", "Scenario Template"],
        examples: &["Examples", "Scenarios"],
        given: &["* ", "Given "],
        when: &["* ", "When "],
        then: &["* ", "Then "],
        and: &["* ", "And "],
        but: &["* ", "But "],
    };

    #[cfg(feature = "dialects")]
    pub const AF: Dialect = Dialect {
        code: "af",
        name: "Afrikaans",
        native: "Afrikaans",
        feature: &["Funksie", "Besigheid Behoefte", "Vermoë"],
        rule: &["Reël", "Reel"],
        background: &["Agtergrond"],
        scenario: &["Voorbeeld", "Situasie"],
        scenario_outline: &["Situasie Uiteensetting"],
        examples: &["Voorbeelde"],
        given: &["* ", "Gegewe "],
        when: &["* ", "Wanneer "],
        then: &["* ", "Dan "],
        and: &["* ", "En "],
        but: &["* ", "Maar "],
    };

    #[cfg(feature = "dialects")]
    pub const AM: Dialect = Dialect {
        code: "am",
        name: "Armenian",
        native: "հայերեն",
        feature: &["Ֆունկցիոնալություն", "Հատկություն"],
        rule: &["Rule"],
        background: &["Կոնտեքստ"],
        scenario: &["Օրինակ", "Սցենար"],
        scenario_outline: &["Սցենարի կառուցվացքը"],
        examples: &["Օրինակներ"],
        given: &["* ", "Դիցուք "],
        when: &["* ", "Եթե ", "Երբ "],
        then: &["* ", "Ապա "],
        and: &["* ", "Եվ "],
        but: &["* ", "Բայց "],
    };

    #[cfg(feature = "dialects")]
    pub const AMH: Dialect = Dialect {
        code: "amh",
        name: "Amharic",
        native: "አማርኛ",
        feature: &["ስራ", "የተፈለገው ስራ", "የሚፈለገው ድርጊት"],
        rule: &["ህግ"],
        background: &["ቅድመ ሁኔታ", "መነሻ", "መነሻ ሀሳብ"],
        scenario: &["ምሳሌ", "ሁናቴ"],
        scenario_outline: &["ሁናቴ ዝርዝር", "ሁናቴ አብነት"],
        examples: &["ምሳሌዎች", "ሁናቴዎች"],
        given: &["* ", "የተሰጠ "],
        when: &["* ", "መቼ "],
        then: &["* ", "ከዚያ "],
        and: &["* ", "እና "],
        but: &["* ", "ግን "],
    };

    #[cfg(feature = "dialects")]
    pub const AN: Dialect = Dialect {
        code: "an",
        name: "Aragonese",
        native: "Aragonés",
        feature: &["Caracteristica"],
        rule: &["Rule"],
        background: &["Antecedents"],
        scenario: &["Eixemplo", "Caso"],
        scenario_outline: &["Esquema del caso"],
        examples: &["Eixemplos"],
        given: &["* ", "Dau ", "Dada ", "Daus ", "Dadas "],
        when: &["* ", "Cuan "],
        then: &["* ", "Alavez ", "Allora ", "Antonces "],
        and: &["* ", "Y ", "E "],
        but: &["* ", "Pero "],
    };

    #[cfg(feature = "dialects")]
    pub const AR: Dialect = Dialect {
        code: "ar",
        name: "Arabic",
        native: "العربية",
        feature: &["خاصية"],
        rule: &["Rule"],
        background: &["الخلفية"],
        scenario: &["مثال", "سيناريو"],
        scenario_outline: &["سيناريو مخطط"],
        examples: &["امثلة"],
        given: &["* ", "بفرض "],
        when: &["* ", "متى ", "عندما "],
        then: &["* ", "اذاً ", "ثم "],
        and: &["* ", "و "],
        but: &["* ", "لكن "],
    };

    #[cfg(feature = "dialects")]
    pub const AST: Dialect = Dialect {
        code: "ast",
        name: "Asturian",
        native: "asturianu",
        feature: &["Carauterística"],
        rule: &["Rule"],
        background: &["Antecedentes"],
        scenario: &["Exemplo", "Casu"],
        scenario_outline: &["Esbozu del casu"],
        examples: &["Exemplos"],
        given: &["* ", "Dáu ", "Dada ", "Daos ", "Daes "],
        when: &["* ", "Cuando "],
        then: &["* ", "Entós "],
        and: &["* ", "Y ", "Ya "],
        but: &["* ", "Peru "],
    };

    #[cfg(feature = "dialects")]
    pub const AZ: Dialect = Dialect {
        code: "az",
        name: "Azerbaijani",
        native: "Azərbaycanca",
        feature: &["Özəllik"],
        rule: &["Rule"],
        background: &["Keçmiş", "Kontekst"],
        scenario: &["Nümunə", "Ssenari"],
        scenario_outline: &["Ssenarinin strukturu"],
        examples: &["Nümunələr"],
        given: &["* ", "Tutaq ki ", "Verilir "],
        when: &["* ", "Əgər ", "Nə vaxt ki "],
        then: &["* ", "O halda "],
        and: &["* ", "Və ", "Həm "],
        but: &["* ", "Amma ", "Ancaq "],
    };

    #[cfg(feature = "dialects")]
    pub const BE: Dialect = Dialect {
        code: "be",
        name: "Belarusian",
        native: "Беларуская",
        feature: &["Функцыянальнасць", "Фіча"],
        rule: &["Правілы"],
        background: &["Кантэкст"],
        scenario: &["Сцэнарый", "Cцэнар"],
        scenario_outline: &["Шаблон сцэнарыя", "Узор сцэнара"],
        examples: &["Прыклады"],
        given: &["* ", "Няхай ", "Дадзена "],
        when: &["* ", "Калі "],
        then: &["* ", "Тады "],
        and: &["* ", "I ", "Ды ", "Таксама "],
        but: &["* ", "Але ", "Інакш "],
    };

    #[cfg(feature = "dialects")]
    pub const BG: Dialect = Dialect {
        code: "bg",
        name: "Bulgarian",
        native: "български",
        feature: &["Функционалност"],
        rule: &["Правило"],
        background: &["Предистория"],
        scenario: &["Пример", "Сценарий"],
        scenario_outline: &["Рамка на сценарий"],
        examples: &["Примери"],
        given: &["* ", "Дадено "],
        when: &["* ", "Когато "],
        then: &["* ", "То "],
        and: &["* ", "И "],
        but: &["* ", "Но "],
    };

    #[cfg(feature = "dialects")]
    pub const BM: Dialect = Dialect {
        code: "bm",
        name: "Malay",
        native: "Bahasa Melayu",
        feature: &["Fungsi"],
        rule: &["Rule"],
        background: &["Latar Belakang"],
        scenario: &["Senario", "Situasi", "Keadaan"],
        scenario_outline: &[
            "Kerangka Senario",
            "Kerangka Situasi",
            "Kerangka Keadaan",
            "Garis Panduan Senario",
        ],
        examples: &["Contoh"],
        given: &["* ", "Diberi ", "Bagi "],
        when: &["* ", "Apabila "],
        then: &["* ", "Maka ", "Kemudian "],
        and: &["* ", "Dan "],
        but: &["* ", "Tetapi ", "Tapi "],
    };

    #[cfg(feature = "dialects")]
    pub const BS: Dialect = Dialect {
        code: "bs",
        name: "Bosnian",
        native: "Bosanski",
        feature: &["Karakteristika"],
        rule: &["Rule"],
        background: &["Pozadina"],
        scenario: &["Primjer", "Scenariju", "Scenario"],
        scenario_outline: &["Scenariju-obris", "Scenario-outline"],
        examples: &["Primjeri"],
        given: &["* ", "Dato "],
        when: &["* ", "Kada "],
        then: &["* ", "Zatim "],
        and: &["* ", "I ", "A "],
        but: &["* ", "Ali "],
    };

    #[cfg(feature = "dialects")]
    pub const CA: Dialect = Dialect {
        code: "ca",
        name: "Catalan",
        native: "català",
        feature: &["Característica", "Funcionalitat"],
        rule: &["Rule"],
        background: &["Rerefons", "Antecedents"],
        scenario: &["Exemple", "Escenari"],
        scenario_outline: &["Esquema de l'escenari"],
        examples: &["Exemples"],
        given: &["* ", "Donat ", "Donada ", "Atès ", "Atesa "],
        when: &["* ", "Quan "],
        then: &["* ", "Aleshores ", "Cal "],
        and: &["* ", "I "],
        but: &["* ", "Però "],
    };

    #[cfg(feature = "dialects")]
    pub const CS: Dialect = Dialect {
        code: "cs",
        name: "Czech",
        native: "Česky",
        feature: &["Požadavek"],
        rule: &["Pravidlo"],
        background: &["Pozadí", "Kontext"],
        scenario: &["Příklad", "Scénář"],
        scenario_outline: &["Náčrt Scénáře", "Osnova scénáře"],
        examples: &["Příklady"],
        given: &["* ", "Pokud ", "Za předpokladu "],
        when: &["* ", "Když "],
        then: &["* ", "Pak "],
        and: &["* ", "A také ", "A "],
        but: &["* ", "Ale "],
    };

    #[cfg(feature = "dialects")]
    pub const CY_GB: Dialect = Dialect {
        code: "cy-GB",
        name: "Welsh",
        native: "Cymraeg",
        feature: &["Arwedd"],
        rule: &["Rule"],
        background: &["Cefndir"],
        scenario: &["Enghraifft", "Scenario"],
        scenario_outline: &["Scenario Amlinellol"],
        examples: &["Enghreifftiau"],
        given: &["* ", "Anrhegedig a "],
        when: &["* ", "Pryd "],
        then: &["* ", "Yna "],
        and: &["* ", "A "],
        but: &["* ", "Ond "],
    };

    #[cfg(feature = "dialects")]
    pub const DA: Dialect = Dialect {
        code: "da",
        name: "Danish",
        native: "dansk",
        feature: &["Egenskab"],
        rule: &["Regel"],
        background: &["Baggrund"],
        scenario: &["Eksempel", "Scenarie"],
        scenario_outline: &["Abstrakt Scenario"],
        examples: &["Eksempler"],
        given: &["* ", "Givet "],
        when: &["* ", "Når "],
        then: &["* ", "Så "],
        and: &["* ", "Og "],
        but: &["* ", "Men "],
    };

    #[cfg(feature = "dialects")]
    pub const DE: Dialect = Dialect {
        code: "de",
        name: "German",
        native: "Deutsch",
        feature: &["Funktionalität", "Funktion"],
        rule: &["Rule", "Regel"],
        background: &[
            "Grundlage",
            "Hintergrund",
            "Voraussetzungen",
            "Vorbedingungen",
        ],
        scenario: &["Beispiel", "Szenario"],
        scenario_outline: &["Szenariogrundriss", "Szenarien"],
        examples: &["Beispiele"],
        given: &["* ", "Angenommen ", "Gegeben sei ", "Gegeben seien "],
        when: &["* ", "Wenn "],
        then: &["* ", "Dann "],
        and: &["* ", "Und "],
        but: &["* ", "Aber "],
    };

    #[cfg(feature = "dialects")]
    pub const EL: Dialect = Dialect {
        code: "el",
        name: "Greek",
        native: "Ελληνικά",
        feature: &["Δυνατότητα", "Λειτουργία"],
        rule: &["Rule"],
        background: &["Υπόβαθρο"],
        scenario: &["Παράδειγμα", "Σενάριο"],
        scenario_outline: &["Περιγραφή Σεναρίου", "Περίγραμμα Σεναρίου"],
        examples: &["Παραδείγματα", "Σενάρια"],
        given: &["* ", "Δεδομένου "],
        when: &["* ", "Όταν "],
        then: &["* ", "Τότε "],
        and: &["* ", "Και "],
        but: &["* ", "Αλλά "],
    };

    #[cfg(feature = "dialects")]
    pub const EM: Dialect = Dialect {
        code: "em",
        name: "Emoji",
        native: "😀",
        feature: &["📚"],
        rule: &["Rule"],
        background: &["💤"],
        scenario: &["🥒", "📕"],
        scenario_outline: &["📖"],
        examples: &["📓"],
        given: &["* ", "😐"],
        when: &["* ", "🎬"],
        then: &["* ", "🙏"],
        and: &["* ", "😂"],
        but: &["* ", "😔"],
    };

    #[cfg(feature = "dialects")]
    pub const EN_SCOUSE: Dialect = Dialect {
        code: "en-Scouse",
        name: "Scouse",
        native: "Scouse",
        feature: &["Feature"],
        rule: &["Rule"],
        background: &["Dis is what went down"],
        scenario: &["The thing of it is"],
        scenario_outline: &["Wharrimean is"],
        examples: &["Examples"],
        given: &["* ", "Givun ", "Youse know when youse got "],
        when: &["* ", "Wun ", "Youse know like when "],
        then: &["* ", "Dun ", "Den youse gotta "],
        and: &["* ", "An "],
        but: &["* ", "Buh "],
    };

    #[cfg(feature = "dialects")]
    pub const EN_AU: Dialect = Dialect {
        code: "en-au",
        name: "Australian",
        native: "Australian",
        feature: &["Pretty much"],
        rule: &["Rule"],
        background: &["First off"],
        scenario: &["Awww, look mate"],
        scenario_outline: &["Reckon it's like"],
        examples: &["You'll wanna"],
        given: &["* ", "Y'know "],
        when: &["* ", "It's just unbelievable "],
        then: &["* ", "But at the end of the day I reckon "],
        and: &["* ", "Too right "],
        but: &["* ", "Yeah nah "],
    };

    #[cfg(feature = "dialects")]
    pub const EN_LOL: Dialect = Dialect {
        code: "en-lol",
        name: "LOLCAT",
        native: "LOLCAT",
        feature: &["OH HAI"],
        rule: &["Rule"],
        background: &["B4"],
        scenario: &["MISHUN"],
        scenario_outline: &["MISHUN SRSLY"],
        examples: &["EXAMPLZ"],
        given: &["* ", "I CAN HAZ "],
        when: &["* ", "WEN "],
        then: &["* ", "DEN "],
        and: &["* ", "AN "],
        but: &["* ", "BUT "],
    };

    #[cfg(feature = "dialects")]
    pub const EN_OLD: Dialect = Dialect {
        code: "en-old",
        name: "Old English",
        native: "Englisc",
        feature: &["Hwaet", "Hwæt"],
        rule: &["Rule"],
        background: &["Aer", "Ær"],
        scenario: &["Swa"],
        scenario_outline: &["Swa hwaer swa", "Swa hwær swa"],
        examples: &["Se the", "Se þe", "Se ðe"],
        given: &["* ", "Thurh ", "Þurh ", "Ðurh "],
        when: &[
            "* ",
            "Bæþsealf ",
            "Bæþsealfa ",
            "Bæþsealfe ",
            "Ciricæw ",
            "Ciricæwe ",
            "Ciricæwa ",
        ],
        then: &["* ", "Tha ", "Þa ", "Ða ", "Tha the ", "Þa þe ", "Ða ðe "],
        and: &["* ", "Ond ", "7 "],
        but: &["* ", "Ac "],
    };

    #[cfg(feature = "dialects")]
    pub const EN_PIRATE: Dialect = Dialect {
        code: "en-pirate",
        name: "Pirate",
        native: "Pirate",
        feature: &["Ahoy matey!"],
        rule: &["Rule"],
        background: &["Yo-ho-ho"],
        scenario: &["Heave to"],
        scenario_outline: &["Shiver me timbers"],
        examples: &["Dead men tell no tales"],
        given: &["* ", "Gangway! "],
        when: &["* ", "Blimey! "],
        then: &["* ", "Let go and haul "],
        and: &["* ", "Aye "],
        but: &["* ", "Avast! "],
    };

    #[cfg(feature = "dialects")]
    pub const EN_TX: Dialect = Dialect {
        code: "en-tx",
        name: "Texas",
        native: "Texas",
        feature: &["This ain’t my first rodeo", "All gussied up"],
        rule: &["Rule"],
        background: &["Lemme tell y'all a story"],
        scenario: &["All hat and no cattle"],
        scenario_outline: &["Serious as a snake bite", "Busy as a hound in flea season"],
        examples: &["Now that's a story longer than a cattle drive in July"],
        given: &["Fixin' to ", "All git out "],
        when: &["Quick out of the chute "],
        then: &["There’s no tree but bears some fruit "],
        and: &["Come hell or high water "],
        but: &["Well now hold on, I'll you what "],
    };

    #[cfg(feature = "dialects")]
    pub const EO: Dialect = Dialect {
        code: "eo",
        name: "Esperanto",
        native: "Esperanto",
        feature: &["Trajto"],
        rule: &["Regulo"],
        background: &["Fono"],
        scenario: &["Ekzemplo", "Scenaro", "Kazo"],
        scenario_outline: &["Konturo de la scenaro", "Skizo", "Kazo-skizo"],
        examples: &["Ekzemploj"],
        given: &["* ", "Donitaĵo ", "Komence "],
        when: &["* ", "Se "],
        then: &["* ", "Do "],
        and: &["* ", "Kaj "],
        but: &["* ", "Sed "],
    };

    #[cfg(feature = "dialects")]
    pub const ES: Dialect = Dialect {
        code: "es",
        name: "Spanish",
        native: "español",
        feature: &["Característica", "Necesidad del negocio", "Requisito"],
        rule: &["Regla", "Regla de negocio"],
        background: &["Antecedentes"],
        scenario: &["Ejemplo", "Escenario"],
        scenario_outline: &["Esquema del escenario"],
        examples: &["Ejemplos"],
        given: &["* ", "Dado ", "Dada ", "Dados ", "Dadas "],
        when: &["* ", "Cuando "],
        then: &["* ", "Entonces "],
        and: &["* ", "Y ", "E "],
        but: &["* ", "Pero "],
    };

    #[cfg(feature = "dialects")]
    pub const ET: Dialect = Dialect {
        code: "et",
        name: "Estonian",
        native: "eesti keel",
        feature: &["Omadus"],
        rule: &["Reegel"],
        background: &["Taust"],
        scenario: &["Juhtum", "Stsenaarium"],
        scenario_outline: &["Raamjuhtum", "Raamstsenaarium"],
        examples: &["Juhtumid"],
        given: &["* ", "Eeldades "],
        when: &["* ", "Kui "],
        then: &["* ", "Siis "],
        and: &["* ", "Ja "],
        but: &["* ", "Kuid "],
    };

    #[cfg(feature = "dialects")]
    pub const FA: Dialect = Dialect {
        code: "fa",
        name: "Persian",
        native: "فارسی",
        feature: &["ویژگی", "قابلیت"],
        rule: &["قانون"],
        background: &["زمینه", "پیش زمینه", "مقدمات"],
        scenario: &["مثال", "سناریو"],
        scenario_outline: &["الگوی سناریو"],
        examples: &["نمونه ها"],
        given: &["* ", "با فرض ", "فرض کنید ", "با در نظر گرفتن "],
        when: &["* ", "هنگامی ", "وقتی "],
        then: &["* ", "آنگاه ", "سپس ", "انتظار می رود "],
        and: &["* ", "و "],
        but: &["* ", "اما "],
    };

    #[cfg(feature = "dialects")]
    pub const FI: Dialect = Dialect {
        code: "fi",
        name: "Finnish",
        native: "suomi",
        feature: &["Ominaisuus"],
        rule: &["Rule"],
        background: &["Tausta"],
        scenario: &["Tapaus"],
        scenario_outline: &["Tapausaihio"],
        examples: &["Tapaukset"],
        given: &["* ", "Oletetaan "],
        when: &["* ", "Kun "],
        then: &["* ", "Niin "],
        and: &["* ", "Ja "],
        but: &["* ", "Mutta "],
    };

    #[cfg(feature = "dialects")]
    pub const FR: Dialect = Dialect {
        code: "fr",
        name: "French",
        native: "français",
        feature: &["Fonctionnalité"],
        rule: &["Règle"],
        background: &["Contexte"],
        scenario: &["Exemple", "Scénario"],
        scenario_outline: &["Plan du scénario", "Plan du Scénario"],
        examples: &["Exemples"],
        given: &[
            "* ",
            "Soit ",
            "Sachant que ",
            "Sachant qu'",
            "Sachant ",
            "Etant donné que ",
            "Etant donné qu'",
            "Etant donné ",
            "Etant donnée ",
            "Etant donnés ",
            "Etant données ",
            "Étant donné que ",
            "Étant donné qu'",
            "Étant donné ",
            "Étant donnée ",
            "Étant donnés ",
            "Étant données ",
        ],
        when: &["* ", "Quand ", "Lorsque ", "Lorsqu'"],
        then: &["* ", "Alors ", "Donc "],
        and: &["* ", "Et que ", "Et qu'", "Et "],
        but: &["* ", "Mais que ", "Mais qu'", "Mais "],
    };

    #[cfg(feature = "dialects")]
    pub const GA: Dialect = Dialect {
        code: "ga",
        name: "Irish",
        native: "Gaeilge",
        feature: &["Gné"],
        rule: &["Riail"],
        background: &["Cúlra"],
        scenario: &["Sampla", "Cás"],
        scenario_outline: &["Cás Achomair"],
        examples: &["Samplaí"],
        given: &[
            "* ",
            "Cuir i gcás go ",
            "Cuir i gcás nach ",
            "Cuir i gcás gur ",
            "Cuir i gcás nár ",
        ],
        when: &["* ", "Nuair a ", "Nuair nach ", "Nuair ba ", "Nuair nár "],
        then: &["* ", "Ansin "],
        and: &["* ", "Agus "],
        but: &["* ", "Ach "],
    };

    #[cfg(feature = "dialects")]
    pub const GJ: Dialect = Dialect {
        code: "gj",
        name: "Gujarati",
        native: "ગુજરાતી",
        feature: &["લક્ષણ", "વ્યાપાર જરૂર", "ક્ષમતા"],
        rule: &["નિયમ"],
        background: &["બેકગ્રાઉન્ડ"],
        scenario: &["ઉદાહરણ", "સ્થિતિ"],
        scenario_outline: &["પરિદ્દશ્ય રૂપરેખા", "પરિદ્દશ્ય ઢાંચો"],
        examples: &["ઉદાહરણો"],
        given: &["* ", "આપેલ છે "],
        when: &["* ", "ક્યારે "],
        then: &["* ", "પછી "],
        and: &["* ", "અને "],
        but: &["* ", "પણ "],
    };

    #[cfg(feature = "dialects")]
    pub const GL: Dialect = Dialect {
        code: "gl",
        name: "Galician",
        native: "galego",
        feature: &["Característica"],
        rule: &["Rule"],
        background: &["Contexto"],
        scenario: &["Exemplo", "Escenario"],
        scenario_outline: &["Esbozo do escenario"],
        examples: &["Exemplos"],
        given: &["* ", "Dado ", "Dada ", "Dados ", "Dadas "],
        when: &["* ", "Cando "],
        then: &["* ", "Entón ", "Logo "],
        and: &["* ", "E "],
        but: &["* ", "Mais ", "Pero "],
    };

    #[cfg(feature = "dialects")]
    pub const HE: Dialect = Dialect {
        code: "he",
        name: "Hebrew",
        native: "עברית",
        feature: &["תכונה"],
        rule: &["כלל"],
        background: &["רקע"],
        scenario: &["דוגמא", "תרחיש"],
        scenario_outline: &["תבנית תרחיש"],
        examples: &["דוגמאות"],
        given: &["* ", "בהינתן "],
        when: &["* ", "כאשר "],
        then: &["* ", "אז ", "אזי "],
        and: &["* ", "וגם "],
        but: &["* ", "אבל "],
    };

    #[cfg(feature = "dialects")]
    pub const HI: Dialect = Dialect {
        code: "hi",
        name: "Hindi",
        native: "हिंदी",
        feature: &["रूप लेख"],
        rule: &["नियम"],
        background: &["पृष्ठभूमि"],
        scenario: &["परिदृश्य"],
        scenario_outline: &["परिदृश्य रूपरेखा"],
        examples: &["उदाहरण"],
        given: &["* ", "अगर ", "यदि ", "चूंकि "],
        when: &["* ", "जब ", "कदा "],
        then: &["* ", "तब ", "तदा "],
        and: &["* ", "और ", "तथा "],
        but: &["* ", "पर ", "परन्तु ", "किन्तु "],
    };

    #[cfg(feature = "dialects")]
    pub const HR: Dialect = Dialect {
        code: "hr",
        name: "Croatian",
        native: "hrvatski",
        feature: &["Osobina", "Mogućnost", "Mogucnost"],
        rule: &["Rule"],
        background: &["Pozadina"],
        scenario: &["Primjer", "Scenarij"],
        scenario_outline: &["Skica", "Koncept"],
        examples: &["Primjeri", "Scenariji"],
        given: &["* ", "Zadan ", "Zadani ", "Zadano ", "Ukoliko "],
        when: &["* ", "Kada ", "Kad "],
        then: &["* ", "Onda "],
        and: &["* ", "I "],
        but: &["* ", "Ali "],
    };

    #[cfg(feature = "dialects")]
    pub const HT: Dialect = Dialect {
        code: "ht",
        name: "Creole",
        native: "kreyòl",
        feature: &["Karakteristik", "Mak", "Fonksyonalite"],
        rule: &["Rule"],
        background: &["Kontèks", "Istorik"],
        scenario: &["Senaryo"],
        scenario_outline: &[
            "Plan senaryo",
            "Plan Senaryo",
            "Senaryo deskripsyon",
            "Senaryo Deskripsyon",
            "Dyagram senaryo",
            "Dyagram Senaryo",
        ],
        examples: &["Egzanp"],
        given: &["* ", "Sipoze ", "Sipoze ke ", "Sipoze Ke "],
        when: &["* ", "Lè ", "Le "],
        then: &["* ", "Lè sa a ", "Le sa a "],
        and: &["* ", "Ak ", "Epi ", "E "],
        but: &["* ", "Men "],
    };

    #[cfg(feature = "dialects")]
    pub const HU: Dialect = Dialect {
        code: "hu",
        name: "Hungarian",
        native: "magyar",
        feature: &["Jellemző"],
        rule: &["Szabály"],
        background: &["Háttér"],
        scenario: &["Példa", "Forgatókönyv"],
        scenario_outline: &["Forgatókönyv vázlat"],
        examples: &["Példák"],
        given: &["* ", "Amennyiben ", "Adott "],
        when: &["* ", "Majd ", "Ha ", "Amikor "],
        then: &["* ", "Akkor "],
        and: &["* ", "És "],
        but: &["* ", "De "],
    };

    #[cfg(feature = "dialects")]
    pub const ID: Dialect = Dialect {
        code: "id",
        name: "Indonesian",
        native: "Bahasa Indonesia",
        feature: &["Fitur"],
        rule: &["Rule", "Aturan"],
        background: &["Dasar", "Latar Belakang"],
        scenario: &["Skenario"],
        scenario_outline: &["Skenario konsep", "Garis-Besar Skenario"],
        examples: &["Contoh", "Misal"],
        given: &[
            "* ",
            "Dengan ",
            "Diketahui ",
            "Diasumsikan ",
            "Bila ",
            "Jika ",
        ],
        when: &["* ", "Ketika "],
        then: &["* ", "Maka ", "Kemudian "],
        and: &["* ", "Dan "],
        but: &["* ", "Tapi ", "Tetapi "],
    };

    #[cfg(feature = "dialects")]
    pub const IS: Dialect = Dialect {
        code: "is",
        name: "Icelandic",
        native: "Íslenska",
        feature: &["Eiginleiki"],
        rule: &["Rule"],
        background: &["Bakgrunnur"],
        scenario: &["Atburðarás"],
        scenario_outline: &["Lýsing Atburðarásar", "Lýsing Dæma"],
        examples: &["Dæmi", "Atburðarásir"],
        given: &["* ", "Ef "],
        when: &["* ", "Þegar "],
        then: &["* ", "Þá "],
        and: &["* ", "Og "],
        but: &["* ", "En "],
    };

    #[cfg(feature = "dialects")]
    pub const IT: Dialect = Dialect {
        code: "it",
        name: "Italian",
        native: "italiano",
        feature: &["Funzionalità", "Esigenza di Business", "Abilità"],
        rule: &["Regola"],
        background: &["Contesto"],
        scenario: &["Esempio", "Scenario"],
        scenario_outline: &["Schema dello scenario"],
        examples: &["Esempi"],
        given: &["* ", "Dato ", "Data ", "Dati ", "Date "],
        when: &["* ", "Quando "],
        then: &["* ", "Allora "],
        and: &["* ", "E ", "Ed "],
        but: &["* ", "Ma "],
    };

    #[cfg(feature = "dialects")]
    pub const JA: Dialect = Dialect {
        code: "ja",
        name: "Japanese",
        native: "日本語",
        feature: &["フィーチャ", "機能"],
        rule: &["ルール"],
        background: &["背景"],
        scenario: &["シナリオ"],
        scenario_outline: &[
            "シナリオアウトライン",
            "シナリオテンプレート",
            "テンプレ",
            "シナリオテンプレ",
        ],
        examples: &["例", "サンプル"],
        given: &["* ", "前提"],
        when: &["* ", "もし"],
        then: &["* ", "ならば"],
        and: &["* ", "且つ", "かつ"],
        but: &["* ", "然し", "しかし", "但し", "ただし"],
    };

    #[cfg(feature = "dialects")]
    pub const JV: Dialect = Dialect {
        code: "jv",
        name: "Javanese",
        native: "Basa Jawa",
        feature: &["Fitur"],
        rule: &["Rule"],
        background: &["Dasar"],
        scenario: &["Skenario"],
        scenario_outline: &["Konsep skenario"],
        examples: &["Conto", "Contone"],
        given: &["* ", "Nalika ", "Nalikaning "],
        when: &["* ", "Manawa ", "Menawa "],
        then: &["* ", "Njuk ", "Banjur "],
        and: &["* ", "Lan "],
        but: &["* ", "Tapi ", "Nanging ", "Ananging "],
    };

    #[cfg(feature = "dialects")]
    pub const KA: Dialect = Dialect {
        code: "ka",
        name: "Georgian",
        native: "ქართული",
        feature: &["თვისება", "მოთხოვნა"],
        rule: &["წესი"],
        background: &["კონტექსტი"],
        scenario: &["მაგალითად", "მაგალითი", "მაგ", "სცენარი"],
        scenario_outline: &["სცენარის ნიმუში", "სცენარის შაბლონი", "ნიმუში", "შაბლონი"],
        examples: &["მაგალითები"],
        given: &["* ", "მოცემული ", "მოცემულია ", "ვთქვათ "],
        when: &["* ", "როდესაც ", "როცა ", "როგორც კი ", "თუ "],
        then: &["* ", "მაშინ "],
        and: &["* ", "და ", "ასევე "],
        but: &["* ", "მაგრამ ", "თუმცა "],
    };

    #[cfg(feature = "dialects")]
    pub const KN: Dialect = Dialect {
        code: "kn",
        name: "Kannada",
        native: "ಕನ್ನಡ",
        feature: &["ಹೆಚ್ಚಳ"],
        rule: &["Rule"],
        background: &["ಹಿನ್ನೆಲೆ"],
        scenario: &["ಉದಾಹರಣೆ", "ಕಥಾಸಾರಾಂಶ"],
        scenario_outline: &["ವಿವರಣೆ"],
        examples: &["ಉದಾಹರಣೆಗಳು"],
        given: &["* ", "ನೀಡಿದ "],
        when: &["* ", "ಸ್ಥಿತಿಯನ್ನು "],
        then: &["* ", "ನಂತರ "],
        and: &["* ", "ಮತ್ತು "],
        but: &["* ", "ಆದರೆ "],
    };

    #[cfg(feature = "dialects")]
    pub const KO: Dialect = Dialect {
        code: "ko",
        name: "Korean",
        native: "한국어",
        feature: &["기능"],
        rule: &["규칙"],
        background: &["배경"],
        scenario: &["시나리오"],
        scenario_outline: &["시나리오 개요"],
        examples: &["예"],
        given: &["* ", "조건 ", "먼저 "],
        when: &["* ", "만일 ", "만약 "],
        then: &["* ", "그러면 "],
        and: &["* ", "그리고 "],
        but: &["* ", "하지만 ", "단 "],
    };

    #[cfg(feature = "dialects")]
    pub const LT: Dialect = Dialect {
        code: "lt",
        name: "Lithuanian",
        native: "lietuvių kalba",
        feature: &["Savybė"],
        rule: &["Rule"],
        background: &["Kontekstas"],
        scenario: &["Pavyzdys", "Scenarijus"],
        scenario_outline: &["Scenarijaus šablonas"],
        examples: &["Pavyzdžiai", "Scenarijai", "Variantai"],
        given: &["* ", "Duota "],
        when: &["* ", "Kai "],
        then: &["* ", "Tada "],
        and: &["* ", "Ir "],
        but: &["* ", "Bet "],
    };

    #[cfg(feature = "dialects")]
    pub const LU: Dialect = Dialect {
        code: "lu",
        name: "Luxemburgish",
        native: "Lëtzebuergesch",
        feature: &["Funktionalitéit"],
        rule: &["Rule"],
        background: &["Hannergrond"],
        scenario: &["Beispill", "Szenario"],
        scenario_outline: &["Plang vum Szenario"],
        examples: &["Beispiller"],
        given: &["* ", "ugeholl "],
        when: &["* ", "wann "],
        then: &["* ", "dann "],
        and: &["* ", "an ", "a "],
        but: &["* ", "awer ", "mä "],
    };

    #[cfg(feature = "dialects")]
    pub const LV: Dialect = Dialect {
        code: "lv",
        name: "Latvian",
        native: "latviešu",
        feature: &["Funkcionalitāte", "Fīča"],
        rule: &["Rule"],
        background: &["Konteksts", "Situācija"],
        scenario: &["Piemērs", "Scenārijs"],
        scenario_outline: &["Scenārijs pēc parauga"],
        examples: &["Piemēri", "Paraugs"],
        given: &["* ", "Kad "],
        when: &["* ", "Ja "],
        then: &["* ", "Tad "],
        and: &["* ", "Un "],
        but: &["* ", "Bet "],
    };

    #[cfg(feature = "dialects")]
    pub const MK_CYRL: Dialect = Dialect {
        code: "mk-Cyrl",
        name: "Macedonian",
        native: "Македонски",
        feature: &["Функционалност", "Бизнис потреба", "Можност"],
        rule: &["Rule"],
        background: &["Контекст", "Содржина"],
        scenario: &["Пример", "Сценарио", "На пример"],
        scenario_outline: &["Преглед на сценарија", "Скица", "Концепт"],
        examples: &["Примери", "Сценарија"],
        given: &["* ", "Дадено ", "Дадена "],
        when: &["* ", "Кога "],
        then: &["* ", "Тогаш "],
        and: &["* ", "И "],
        but: &["* ", "Но "],
    };

    #[cfg(feature = "dialects")]
    pub const MK_LATN: Dialect = Dialect {
        code: "mk-Latn",
        name: "Macedonian (Latin)",
        native: "Makedonski (Latinica)",
        feature: &["Funkcionalnost", "Biznis potreba", "Mozhnost"],
        rule: &["Rule"],
        background: &["Kontekst", "Sodrzhina"],
        scenario: &["Scenario", "Na primer"],
        scenario_outline: &["Pregled na scenarija", "Skica", "Koncept"],
        examples: &["Primeri", "Scenaria"],
        given: &["* ", "Dadeno ", "Dadena "],
        when: &["* ", "Koga "],
        then: &["* ", "Togash "],
        and: &["* ", "I "],
        but: &["* ", "No "],
    };

    #[cfg(feature = "dialects")]
    pub const ML: Dialect = Dialect {
        code: "ml",
        name: "Malayalam",
        native: "മലയാളം",
        feature: &["സവിശേഷത"],
        rule: &["നിയമം"],
        background: &["പശ്ചാത്തലം"],
        scenario: &["രംഗം"],
        scenario_outline: &["സാഹചര്യത്തിന്റെ രൂപരേഖ"],
        examples: &["ഉദാഹരണങ്ങൾ"],
        given: &["* ", "നൽകിയത്"],
        when: &["എപ്പോൾ"],
        then: &["* ", "പിന്നെ"],
        and: &["* ", "ഒപ്പം"],
        but: &["* ", "പക്ഷേ"],
    };

    #[cfg(feature = "dialects")]
    pub const MN: Dialect = Dialect {
        code: "mn",
        name: "Mongolian",
        native: "монгол",
        feature: &["Функц", "Функционал"],
        rule: &["Rule"],
        background: &["Агуулга"],
        scenario: &["Сценар"],
        scenario_outline: &["Сценарын төлөвлөгөө"],
        examples: &["Тухайлбал"],
        given: &["* ", "Өгөгдсөн нь ", "Анх "],
        when: &["* ", "Хэрэв "],
        then: &["* ", "Тэгэхэд ", "Үүний дараа "],
        and: &["* ", "Мөн ", "Тэгээд "],
        but: &["* ", "Гэхдээ ", "Харин "],
    };

    #[cfg(feature = "dialects")]
    pub const MR: Dialect = Dialect {
        code: "mr",
        name: "Marathi",
        native: "मराठी",
        feature: &["वैशिष्ट्य", "सुविधा"],
        rule: &["नियम"],
        background: &["पार्श्वभूमी"],
        scenario: &["परिदृश्य"],
        scenario_outline: &["परिदृश्य रूपरेखा"],
        examples: &["उदाहरण"],
        given: &["* ", "जर", "दिलेल्या प्रमाणे "],
        when: &["* ", "जेव्हा "],
        then: &["* ", "मग ", "तेव्हा "],
        and: &["* ", "आणि ", "तसेच "],
        but: &["* ", "पण ", "परंतु "],
    };

    #[cfg(feature = "dialects")]
    pub const NE: Dialect = Dialect {
        code: "ne",
        name: "Nepali",
        native: "नेपाली",
        feature: &["सुविधा", "विशेषता"],
        rule: &["नियम"],
        background: &["पृष्ठभूमी"],
        scenario: &["परिदृश्य"],
        scenario_outline: &["परिदृश्य रूपरेखा"],
        examples: &["उदाहरण", "उदाहरणहरु"],
        given: &["* ", "दिइएको ", "दिएको ", "यदि "],
        when: &["* ", "जब "],
        then: &["* ", "त्यसपछि ", "अनी "],
        and: &["* ", "र ", "अनि "],
        but: &["* ", "तर "],
    };

    #[cfg(feature = "dialects")]
    pub const NL: Dialect = Dialect {
        code: "nl",
        name: "Dutch",
        native: "Nederlands",
        feature: &["Functionaliteit"],
        rule: &["Regel"],
        background: &["Achtergrond"],
        scenario: &["Voorbeeld", "Scenario"],
        scenario_outline: &["Abstract Scenario"],
        examples: &["Voorbeelden"],
        given: &["* ", "Gegeven ", "Stel "],
        when: &["* ", "Als ", "Wanneer "],
        then: &["* ", "Dan "],
        and: &["* ", "En "],
        but: &["* ", "Maar "],
    };

    #[cfg(feature = "dialects")]
    pub const NO: Dialect = Dialect {
        code: "no",
        name: "Norwegian",
        native: "norsk",
        feature: &["Egenskap"],
        rule: &["Regel"],
        background: &["Bakgrunn"],
        scenario: &["Eksempel", "Scenario"],
        scenario_outline: &["Scenariomal", "Abstrakt Scenario"],
        examples: &["Eksempler"],
        given: &["* ", "Gitt "],
        when: &["* ", "Når "],
        then: &["* ", "Så "],
        and: &["* ", "Og "],
        but: &["* ", "Men "],
    };

    #[cfg(feature = "dialects")]
    pub const PA: Dialect = Dialect {
        code: "pa",
        name: "Panjabi",
        native: "ਪੰਜਾਬੀ",
        feature: &["ਖਾਸੀਅਤ", "ਮੁਹਾਂਦਰਾ", "ਨਕਸ਼ ਨੁਹਾਰ"],
        rule: &["Rule"],
        background: &["ਪਿਛੋਕੜ"],
        scenario: &["ਉਦਾਹਰਨ", "ਪਟਕਥਾ"],
        scenario_outline: &["ਪਟਕਥਾ ਢਾਂਚਾ", "ਪਟਕਥਾ ਰੂਪ ਰੇਖਾ"],
        examples: &["ਉਦਾਹਰਨਾਂ"],
        given: &["* ", "ਜੇਕਰ ", "ਜਿਵੇਂ ਕਿ "],
        when: &["* ", "ਜਦੋਂ "],
        then: &["* ", "ਤਦ "],
        and: &["* ", "ਅਤੇ "],
        but: &["* ", "ਪਰ "],
    };

    #[cfg(feature = "dialects")]
    pub const PL: Dialect = Dialect {
        code: "pl",
        name: "Polish",
        native: "polski",
        feature: &["Właściwość", "Funkcja", "Aspekt", "Potrzeba biznesowa"],
        rule: &["Zasada", "Reguła"],
        background: &["Założenia"],
        scenario: &["Przykład", "Scenariusz"],
        scenario_outline: &["Szablon scenariusza"],
        examples: &["Przykłady"],
        given: &["* ", "Zakładając ", "Mając ", "Zakładając, że "],
        when: &["* ", "Jeżeli ", "Jeśli ", "Gdy ", "Kiedy "],
        then: &["* ", "Wtedy "],
        and: &["* ", "Oraz ", "I "],
        but: &["* ", "Ale "],
    };

    #[cfg(feature = "dialects")]
    pub const PT: Dialect = Dialect {
        code: "pt",
        name: "Portuguese",
        native: "português",
        feature: &["Funcionalidade", "Característica", "Caracteristica"],
        rule: &["Regra"],
        background: &["Contexto", "Cenário de Fundo", "Cenario de Fundo", "Fundo"],
        scenario: &["Exemplo", "Cenário", "Cenario"],
        scenario_outline: &[
            "Esquema do Cenário",
            "Esquema do Cenario",
            "Delineação do Cenário",
            "Delineacao do Cenario",
        ],
        examples: &["Exemplos", "Cenários", "Cenarios"],
        given: &["* ", "Dado ", "Dada ", "Dados ", "Dadas "],
        when: &["* ", "Quando "],
        then: &["* ", "Então ", "Entao "],
        and: &["* ", "E "],
        but: &["* ", "Mas "],
    };

    #[cfg(feature = "dialects")]
    pub const RO: Dialect = Dialect {
        code: "ro",
        name: "Romanian",
        native: "română",
        feature: &["Functionalitate", "Funcționalitate", "Funcţionalitate"],
        rule: &["Rule"],
        background: &["Context"],
        scenario: &["Exemplu", "Scenariu"],
        scenario_outline: &["Structura scenariu", "Structură scenariu"],
        examples: &["Exemple"],
        given: &[
            "* ",
            "Date fiind ",
            "Dat fiind ",
            "Dată fiind",
            "Dati fiind ",
            "Dați fiind ",
            "Daţi fiind ",
        ],
        when: &["* ", "Cand ", "Când "],
        then: &["* ", "Atunci "],
        and: &["* ", "Si ", "Și ", "Şi "],
        but: &["* ", "Dar "],
    };

    #[cfg(feature = "dialects")]
    pub const RU: Dialect = Dialect {
        code: "ru",
        name: "Russian",
        native: "русский",
        feature: &[
            "Функция",
            "Функциональность",
            "Функционал",
            "Свойство",
            "Фича",
        ],
        rule: &["Правило"],
        background: &["Предыстория", "Контекст"],
        scenario: &["Пример", "Сценарий"],
        scenario_outline: &["Структура сценария", "Шаблон сценария"],
        examples: &["Примеры", "Значения"],
        given: &["* ", "Допустим ", "Дано ", "Пусть "],
        when: &["* ", "Когда ", "Если "],
        then: &["* ", "То ", "Затем ", "Тогда "],
        and: &["* ", "И ", "К тому же ", "Также "],
        but: &["* ", "Но ", "А ", "Иначе "],
    };

    #[cfg(feature = "dialects")]
    pub const SK: Dialect = Dialect {
        code: "sk",
        name: "Slovak",
        native: "Slovensky",
        feature: &["Požiadavka", "Funkcia", "Vlastnosť"],
        rule: &["Rule"],
        background: &["Pozadie"],
        scenario: &["Príklad", "Scenár"],
        scenario_outline: &["Náčrt Scenáru", "Náčrt Scenára", "Osnova Scenára"],
        examples: &["Príklady"],
        given: &["* ", "Pokiaľ ", "Za predpokladu "],
        when: &["* ", "Keď ", "Ak "],
        then: &["* ", "Tak ", "Potom "],
        and: &["* ", "A ", "A tiež ", "A taktiež ", "A zároveň "],
        but: &["* ", "Ale "],
    };

    #[cfg(feature = "dialects")]
    pub const SL: Dialect = Dialect {
        code: "sl",
        name: "Slovenian",
        native: "Slovenski",
        feature: &[
            "Funkcionalnost",
            "Funkcija",
            "Možnosti",
            "Moznosti",
            "Lastnost",
            "Značilnost",
        ],
        rule: &["Rule"],
        background: &["Kontekst", "Osnova", "Ozadje"],
        scenario: &["Primer", "Scenarij"],
        scenario_outline: &[
            "Struktura scenarija",
            "Skica",
            "Koncept",
            "Oris scenarija",
            "Osnutek",
        ],
        examples: &["Primeri", "Scenariji"],
        given: &["Dano ", "Podano ", "Zaradi ", "Privzeto "],
        when: &["Ko ", "Ce ", "Če ", "Kadar "],
        then: &["Nato ", "Potem ", "Takrat "],
        and: &["In ", "Ter "],
        but: &["Toda ", "Ampak ", "Vendar "],
    };

    #[cfg(feature = "dialects")]
    pub const SR_CYRL: Dialect = Dialect {
        code: "sr-Cyrl",
        name: "Serbian",
        native: "Српски",
        feature: &["Функционалност", "Могућност", "Особина"],
        rule: &["Правило"],
        background: &["Контекст", "Основа", "Позадина"],
        scenario: &["Сценарио", "Пример"],
        scenario_outline: &["Структура сценарија", "Скица", "Концепт"],
        examples: &["Примери", "Сценарији"],
        given: &["* ", "За дато ", "За дате ", "За дати "],
        when: &["* ", "Када ", "Кад "],
        then: &["* ", "Онда "],
        and: &["* ", "И "],
        but: &["* ", "Али "],
    };

    #[cfg(feature = "dialects")]
    pub const SR_LATN: Dialect = Dialect {
        code: "sr-Latn",
        name: "Serbian (Latin)",
        native: "Srpski (Latinica)",
        feature: &["Funkcionalnost", "Mogućnost", "Mogucnost", "Osobina"],
        rule: &["Pravilo"],
        background: &["Kontekst", "Osnova", "Pozadina"],
        scenario: &["Scenario", "Primer"],
        scenario_outline: &["Struktura scenarija", "Skica", "Koncept"],
        examples: &["Primeri", "Scenariji"],
        given: &["* ", "Za dato ", "Za date ", "Za dati "],
        when: &["* ", "Kada ", "Kad "],
        then: &["* ", "Onda "],
        and: &["* ", "I "],
        but: &["* ", "Ali "],
    };

    #[cfg(feature = "dialects")]
    pub const SV: Dialect = Dialect {
        code: "sv",
        name: "Swedish",
        native: "Svenska",
        feature: &["Egenskap"],
        rule: &["Regel"],
        background: &["Bakgrund"],
        scenario: &["Scenario"],
        scenario_outline: &["Abstrakt Scenario", "Scenariomall"],
        examples: &["Exempel"],
        given: &["* ", "Givet "],
        when: &["* ", "När "],
        then: &["* ", "Så "],
        and: &["* ", "Och "],
        but: &["* ", "Men "],
    };

    #[cfg(feature = "dialects")]
    pub const TA: Dialect = Dialect {
        code: "ta",
        name: "Tamil",
        native: "தமிழ்",
        feature: &["அம்சம்", "வணிக தேவை", "திறன்"],
        rule: &["Rule"],
        background: &["பின்னணி"],
        scenario: &["உதாரணமாக", "காட்சி"],
        scenario_outline: &["காட்சி சுருக்கம்", "காட்சி வார்ப்புரு"],
        examples: &["எடுத்துக்காட்டுகள்", "காட்சிகள்", "நிலைமைகளில்"],
        given: &["* ", "கொடுக்கப்பட்ட "],
        when: &["* ", "எப்போது "],
        then: &["* ", "அப்பொழுது "],
        and: &["* ", "மேலும் ", "மற்றும் "],
        but: &["* ", "ஆனால் "],
    };

    #[cfg(feature = "dialects")]
    pub const TE: Dialect = Dialect {
        code: "te",
        name: "Telugu",
        native: "తెలుగు",
        feature: &["గుణము"],
        rule: &["Rule"],
        background: &["నేపథ్యం"],
        scenario: &["ఉదాహరణ", "సన్నివేశం"],
        scenario_outline: &["కథనం"],
        examples: &["ఉదాహరణలు"],
        given: &["* ", "చెప్పబడినది "],
        when: &["* ", "ఈ పరిస్థితిలో "],
        then: &["* ", "అప్పుడు "],
        and: &["* ", "మరియు "],
        but: &["* ", "కాని "],
    };

    #[cfg(feature = "dialects")]
    pub const TH: Dialect = Dialect {
        code: "th",
        name: "Thai",
        native: "ไทย",
        feature: &["โครงหลัก", "ความต้องการทางธุรกิจ", "ความสามารถ"],
        rule: &["Rule"],
        background: &["แนวคิด"],
        scenario: &["เหตุการณ์"],
        scenario_outline: &["สรุปเหตุการณ์", "โครงสร้างของเหตุการณ์"],
        examples: &["ชุดของตัวอย่าง", "ชุดของเหตุการณ์"],
        given: &["* ", "กำหนดให้ "],
        when: &["* ", "เมื่อ "],
        then: &["* ", "ดังนั้น "],
        and: &["* ", "และ "],
        but: &["* ", "แต่ "],
    };

    #[cfg(feature = "dialects")]
    pub const TLH: Dialect = Dialect {
        code: "tlh",
        name: "Klingon",
        native: "tlhIngan",
        feature: &["Qap", "Qu'meH 'ut", "perbogh", "poQbogh malja'", "laH"],
        rule: &["Rule"],
        background: &["mo'"],
        scenario: &["lut"],
        scenario_outline: &["lut chovnatlh"],
        examples: &["ghantoH", "lutmey"],
        given: &["* ", "ghu' noblu' ", "DaH ghu' bejlu' "],
        when: &["* ", "qaSDI' "],
        then: &["* ", "vaj "],
        and: &["* ", "'ej ", "latlh "],
        but: &["* ", "'ach ", "'a "],
    };

    #[cfg(feature = "dialects")]
    pub const TR: Dialect = Dialect {
        code: "tr",
        name: "Turkish",
        native: "Türkçe",
        feature: &[
            "Özellik",
            "İş Gereksinimi",
            "Gereksinim",
            "İşlev",
            "Kullanıcı Hikayesi",
            "Yetenek",
            "Teknik Gereksinim",
        ],
        rule: &["Kural", "İş Kuralı", "Kaide", "Hüküm", "Madde"],
        background: &[
            "Geçmiş",
            "Arka Plan",
            "Ön Koşul",
            "Önkoşul",
            "Önceki Durum",
            "Giriş",
            "Mukaddime",
            "Mevcut Durum",
        ],
        scenario: &["Örnek", "Senaryo", "Durum", "Vaka"],
        scenario_outline: &["Senaryo taslağı", "Senaryo şablonu"],
        examples: &["Örnekler", "Değerler"],
        given: &[
            "* ",
            "Mevcut ",
            "Önceden ",
            "Geçmişte ",
            "Daha önce ",
            "Halihazırda ",
            "Zaten ",
            "Sistemde ",
            "Diyelim ki ",
            "Varsayalım ki ",
            "Farz edelim ki ",
            "Kabul edelim ki ",
            "Başlangıçta ",
            "Varsayılan olarak ",
            "Biliniyor ki ",
        ],
        when: &[
            "* ",
            "Eğer ",
            "Eğer ki ",
            "Ne zaman ",
            "Ne zaman ki ",
            "Şayet ",
        ],
        then: &[
            "* ",
            "Beklenen ",
            "O zaman ",
            "Sonuç olarak ",
            "Böylece ",
            "Bunun üzerine ",
            "Bu durumda ",
            "O takdirde ",
            "Şu halde ",
            "Netice itibariyle ",
            "Buna binaen ",
        ],
        and: &[
            "* ",
            "Ve ",
            "Hem de ",
            "Bir de ",
            "Ayrıca ",
            "İlaveten ",
            "Buna ek olarak ",
        ],
        but: &[
            "* ",
            "Fakat ",
            "Ama ",
            "Ancak ",
            "Yalnız ",
            "Lakin ",
            "Meğer ki ",
            "Buna mukabil ",
            "Aksi halde ",
        ],
    };

    #[cfg(feature = "dialects")]
    pub const TT: Dialect = Dialect {
        code: "tt",
        name: "Tatar",
        native: "Татарча",
        feature: &["Мөмкинлек", "Үзенчәлеклелек"],
        rule: &["Rule"],
        background: &["Кереш"],
        scenario: &["Сценарий"],
        scenario_outline: &["Сценарийның төзелеше"],
        examples: &["Үрнәкләр", "Мисаллар"],
        given: &["* ", "Әйтик "],
        when: &["* ", "Әгәр "],
        then: &["* ", "Нәтиҗәдә "],
        and: &["* ", "Һәм ", "Вә "],
        but: &["* ", "Ләкин ", "Әмма "],
    };

    #[cfg(feature = "dialects")]
    pub const UK: Dialect = Dialect {
        code: "uk",
        name: "Ukrainian",
        native: "Українська",
        feature: &["Функціонал"],
        rule: &["Rule"],
        background: &["Передумова"],
        scenario: &["Приклад", "Сценарій"],
        scenario_outline: &["Структура сценарію"],
        examples: &["Приклади"],
        given: &["* ", "Припустимо ", "Припустимо, що ", "Нехай ", "Дано "],
        when: &["* ", "Якщо ", "Коли "],
        then: &["* ", "То ", "Тоді "],
        and: &["* ", "І ", "А також ", "Та "],
        but: &["* ", "Але "],
    };

    #[cfg(feature = "dialects")]
    pub const UR: Dialect = Dialect {
        code: "ur",
        name: "Urdu",
        native: "اردو",
        feature: &["صلاحیت", "کاروبار کی ضرورت", "خصوصیت"],
        rule: &["Rule"],
        background: &["پس منظر"],
        scenario: &["منظرنامہ"],
        scenario_outline: &["منظر نامے کا خاکہ"],
        examples: &["مثالیں"],
        given: &["* ", "اگر ", "بالفرض ", "فرض کیا "],
        when: &["* ", "جب "],
        then: &["* ", "پھر ", "تب "],
        and: &["* ", "اور "],
        but: &["* ", "لیکن "],
    };

    #[cfg(feature = "dialects")]
    pub const UZ: Dialect = Dialect {
        code: "uz",
        name: "Uzbek",
        native: "Узбекча",
        feature: &["Функционал"],
        rule: &["Rule"],
        background: &["Тарих"],
        scenario: &["Сценарий"],
        scenario_outline: &["Сценарий структураси"],
        examples: &["Мисоллар"],
        given: &["* ", "Belgilangan "],
        when: &["* ", "Агар "],
        then: &["* ", "Унда "],
        and: &["* ", "Ва "],
        but: &["* ", "Лекин ", "Бирок ", "Аммо "],
    };

    #[cfg(feature = "dialects")]
    pub const VI: Dialect = Dialect {
        code: "vi",
        name: "Vietnamese",
        native: "Tiếng Việt",
        feature: &["Tính năng"],
        rule: &["Quy tắc"],
        background: &["Bối cảnh"],
        scenario: &["Tình huống", "Kịch bản"],
        scenario_outline: &["Khung tình huống", "Khung kịch bản"],
        examples: &["Dữ liệu"],
        given: &["* ", "Biết ", "Cho "],
        when: &["* ", "Khi "],
        then: &["* ", "Thì "],
        and: &["* ", "Và "],
        but: &["* ", "Nhưng "],
    };

    #[cfg(feature = "dialects")]
    pub const ZH_CN: Dialect = Dialect {
        code: "zh-CN",
        name: "Chinese simplified",
        native: "简体中文",
        feature: &["功能"],
        rule: &["Rule", "规则"],
        background: &["背景"],
        scenario: &["场景", "剧本"],
        scenario_outline: &["场景大纲", "剧本大纲"],
        examples: &["例子"],
        given: &["* ", "假如", "假设", "假定"],
        when: &["* ", "当"],
        then: &["* ", "那么"],
        and: &["* ", "而且", "并且", "同时"],
        but: &["* ", "但是"],
    };

    #[cfg(feature = "dialects")]
    pub const ZH_TW: Dialect = Dialect {
        code: "zh-TW",
        name: "Chinese traditional",
        native: "繁體中文",
        feature: &["功能"],
        rule: &["Rule"],
        background: &["背景"],
        scenario: &["場景", "劇本"],
        scenario_outline: &["場景大綱", "劇本大綱"],
        examples: &["例子"],
        given: &["* ", "假如", "假設", "假定"],
        when: &["* ", "當"],
        then: &["* ", "那麼"],
        and: &["* ", "而且", "並且", "同時"],
        but: &["* ", "但是"],
    };
}

#[cfg(not(feature = "dialects"))]
pub(super) static DIALECTS: &[Dialect] = &[Dialect::EN];

#[cfg(feature = "dialects")]
pub(super) static DIALECTS: &[Dialect] = &[
    Dialect::AF,
    Dialect::AM,
    Dialect::AMH,
    Dialect::AN,
    Dialect::AR,
    Dialect::AST,
    Dialect::AZ,
    Dialect::BE,
    Dialect::BG,
    Dialect::BM,
    Dialect::BS,
    Dialect::CA,
    Dialect::CS,
    Dialect::CY_GB,
    Dialect::DA,
    Dialect::DE,
    Dialect::EL,
    Dialect::EM,
    Dialect::EN,
    Dialect::EN_SCOUSE,
    Dialect::EN_AU,
    Dialect::EN_LOL,
    Dialect::EN_OLD,
    Dialect::EN_PIRATE,
    Dialect::EN_TX,
    Dialect::EO,
    Dialect::ES,
    Dialect::ET,
    Dialect::FA,
    Dialect::FI,
    Dialect::FR,
    Dialect::GA,
    Dialect::GJ,
    Dialect::GL,
    Dialect::HE,
    Dialect::HI,
    Dialect::HR,
    Dialect::HT,
    Dialect::HU,
    Dialect::ID,
    Dialect::IS,
    Dialect::IT,
    Dialect::JA,
    Dialect::JV,
    Dialect::KA,
    Dialect::KN,
    Dialect::KO,
    Dialect::LT,
    Dialect::LU,
    Dialect::LV,
    Dialect::MK_CYRL,
    Dialect::MK_LATN,
    Dialect::ML,
    Dialect::MN,
    Dialect::MR,
    Dialect::NE,
    Dialect::NL,
    Dialect::NO,
    Dialect::PA,
    Dialect::PL,
    Dialect::PT,
    Dialect::RO,
    Dialect::RU,
    Dialect::SK,
    Dialect::SL,
    Dialect::SR_CYRL,
    Dialect::SR_LATN,
    Dialect::SV,
    Dialect::TA,
    Dialect::TE,
    Dialect::TH,
    Dialect::TLH,
    Dialect::TR,
    Dialect::TT,
    Dialect::UK,
    Dialect::UR,
    Dialect::UZ,
    Dialect::VI,
    Dialect::ZH_CN,
    Dialect::ZH_TW,
];
