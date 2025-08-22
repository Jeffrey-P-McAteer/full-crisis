use super::keys::TranslationKey;
use std::collections::HashMap;

/// Translation data structure
#[derive(Debug, Clone)]
pub struct Translation {
    pub key: TranslationKey,
    pub translations: HashMap<String, String>,
}

impl Translation {
    pub fn new(key: TranslationKey) -> Self {
        Self {
            key,
            translations: HashMap::new(),
        }
    }
    
    pub fn add_translation(mut self, lang: &str, text: &str) -> Self {
        self.translations.insert(lang.to_string(), text.to_string());
        self
    }
}

/// Get all builtin translations
pub fn get_builtin_translations() -> Vec<Translation> {
    vec![
        // Main Menu
        Translation::new(TranslationKey::ContinueGame)
            .add_translation("eng", "Continue Game")
            .add_translation("spa", "Continuar Juego")
            .add_translation("fra", "Continuer le Jeu")
            .add_translation("deu", "Spiel Fortsetzen")
            .add_translation("ita", "Continua Gioco")
            .add_translation("por", "Continuar Jogo")
            .add_translation("rus", "Продолжить Игру")
            .add_translation("jpn", "ゲームを続ける")
            .add_translation("kor", "게임 계속하기")
            .add_translation("zho", "继续游戏"),
        
        Translation::new(TranslationKey::NewGame)
            .add_translation("eng", "New Game")
            .add_translation("spa", "Nuevo Juego")
            .add_translation("fra", "Nouveau Jeu")
            .add_translation("deu", "Neues Spiel")
            .add_translation("ita", "Nuovo Gioco")
            .add_translation("por", "Novo Jogo")
            .add_translation("rus", "Новая Игра")
            .add_translation("jpn", "新しいゲーム")
            .add_translation("kor", "새 게임")
            .add_translation("zho", "新游戏"),
        
        Translation::new(TranslationKey::Settings)
            .add_translation("eng", "Settings")
            .add_translation("spa", "Configuración")
            .add_translation("fra", "Paramètres")
            .add_translation("deu", "Einstellungen")
            .add_translation("ita", "Impostazioni")
            .add_translation("por", "Configurações")
            .add_translation("rus", "Настройки")
            .add_translation("jpn", "設定")
            .add_translation("kor", "설정")
            .add_translation("zho", "设置"),
        
        Translation::new(TranslationKey::Licenses)
            .add_translation("eng", "Licenses")
            .add_translation("spa", "Licencias")
            .add_translation("fra", "Licences")
            .add_translation("deu", "Lizenzen")
            .add_translation("ita", "Licenze")
            .add_translation("por", "Licenças")
            .add_translation("rus", "Лицензии")
            .add_translation("jpn", "ライセンス")
            .add_translation("kor", "라이선스")
            .add_translation("zho", "许可证"),
        
        Translation::new(TranslationKey::QuitGame)
            .add_translation("eng", "Quit Game")
            .add_translation("spa", "Salir del Juego")
            .add_translation("fra", "Quitter le Jeu")
            .add_translation("deu", "Spiel Beenden")
            .add_translation("ita", "Esci dal Gioco")
            .add_translation("por", "Sair do Jogo")
            .add_translation("rus", "Выйти из Игры")
            .add_translation("jpn", "ゲームを終了")
            .add_translation("kor", "게임 종료")
            .add_translation("zho", "退出游戏"),
        
        // New Game UI
        Translation::new(TranslationKey::PlayerName)
            .add_translation("eng", "Player Name:")
            .add_translation("spa", "Nombre del Jugador:")
            .add_translation("fra", "Nom du Joueur:")
            .add_translation("deu", "Spielername:")
            .add_translation("ita", "Nome Giocatore:")
            .add_translation("por", "Nome do Jogador:")
            .add_translation("rus", "Имя Игрока:")
            .add_translation("jpn", "プレイヤー名:")
            .add_translation("kor", "플레이어 이름:")
            .add_translation("zho", "玩家姓名:"),
        
        Translation::new(TranslationKey::EnterName)
            .add_translation("eng", "Enter name...")
            .add_translation("spa", "Ingresa nombre...")
            .add_translation("fra", "Entrez le nom...")
            .add_translation("deu", "Namen eingeben...")
            .add_translation("ita", "Inserisci nome...")
            .add_translation("por", "Digite o nome...")
            .add_translation("rus", "Введите имя...")
            .add_translation("jpn", "名前を入力...")
            .add_translation("kor", "이름 입력...")
            .add_translation("zho", "输入姓名..."),
        
        Translation::new(TranslationKey::GameType)
            .add_translation("eng", "Game Type:")
            .add_translation("spa", "Tipo de Juego:")
            .add_translation("fra", "Type de Jeu:")
            .add_translation("deu", "Spieltyp:")
            .add_translation("ita", "Tipo di Gioco:")
            .add_translation("por", "Tipo de Jogo:")
            .add_translation("rus", "Тип Игры:")
            .add_translation("jpn", "ゲームタイプ:")
            .add_translation("kor", "게임 유형:")
            .add_translation("zho", "游戏类型:"),
        
        Translation::new(TranslationKey::SelectGameType)
            .add_translation("eng", "Select game type")
            .add_translation("spa", "Seleccionar tipo de juego")
            .add_translation("fra", "Sélectionner le type de jeu")
            .add_translation("deu", "Spieltyp auswählen")
            .add_translation("ita", "Seleziona tipo di gioco")
            .add_translation("por", "Selecionar tipo de jogo")
            .add_translation("rus", "Выберите тип игры")
            .add_translation("jpn", "ゲームタイプを選択")
            .add_translation("kor", "게임 유형 선택")
            .add_translation("zho", "选择游戏类型"),
        
        Translation::new(TranslationKey::Go)
            .add_translation("eng", "Go")
            .add_translation("spa", "Ir")
            .add_translation("fra", "Aller")
            .add_translation("deu", "Los")
            .add_translation("ita", "Vai")
            .add_translation("por", "Ir")
            .add_translation("rus", "Идти")
            .add_translation("jpn", "開始")
            .add_translation("kor", "시작")
            .add_translation("zho", "开始"),
        
        // Continue Game UI
        Translation::new(TranslationKey::SavedGame)
            .add_translation("eng", "Saved Game:")
            .add_translation("spa", "Juego Guardado:")
            .add_translation("fra", "Jeu Sauvegardé:")
            .add_translation("deu", "Gespeichertes Spiel:")
            .add_translation("ita", "Gioco Salvato:")
            .add_translation("por", "Jogo Salvo:")
            .add_translation("rus", "Сохранённая Игра:")
            .add_translation("jpn", "保存されたゲーム:")
            .add_translation("kor", "저장된 게임:")
            .add_translation("zho", "保存的游戏:"),
        
        Translation::new(TranslationKey::SelectGame)
            .add_translation("eng", "Select game")
            .add_translation("spa", "Seleccionar juego")
            .add_translation("fra", "Sélectionner le jeu")
            .add_translation("deu", "Spiel auswählen")
            .add_translation("ita", "Seleziona gioco")
            .add_translation("por", "Selecionar jogo")
            .add_translation("rus", "Выберите игру")
            .add_translation("jpn", "ゲームを選択")
            .add_translation("kor", "게임 선택")
            .add_translation("zho", "选择游戏"),
        
        Translation::new(TranslationKey::Play)
            .add_translation("eng", "Play")
            .add_translation("spa", "Jugar")
            .add_translation("fra", "Jouer")
            .add_translation("deu", "Spielen")
            .add_translation("ita", "Gioca")
            .add_translation("por", "Jogar")
            .add_translation("rus", "Играть")
            .add_translation("jpn", "プレイ")
            .add_translation("kor", "플레이")
            .add_translation("zho", "开始游戏"),
        
        Translation::new(TranslationKey::Delete)
            .add_translation("eng", "Delete")
            .add_translation("spa", "Eliminar")
            .add_translation("fra", "Supprimer")
            .add_translation("deu", "Löschen")
            .add_translation("ita", "Elimina")
            .add_translation("por", "Excluir")
            .add_translation("rus", "Удалить")
            .add_translation("jpn", "削除")
            .add_translation("kor", "삭제")
            .add_translation("zho", "删除"),
        
        Translation::new(TranslationKey::DeleteGame)
            .add_translation("eng", "Are you sure you want to delete this saved game?")
            .add_translation("spa", "¿Estás seguro de que quieres eliminar esta partida guardada?")
            .add_translation("fra", "Êtes-vous sûr de vouloir supprimer cette partie sauvegardée?")
            .add_translation("deu", "Sind Sie sicher, dass Sie diesen Spielstand löschen möchten?")
            .add_translation("ita", "Sei sicuro di voler eliminare questo gioco salvato?")
            .add_translation("por", "Tem certeza de que deseja excluir este jogo salvo?")
            .add_translation("rus", "Вы уверены, что хотите удалить это сохранение?")
            .add_translation("jpn", "この保存されたゲームを削除しますか？")
            .add_translation("kor", "이 저장된 게임을 삭제하시겠습니까?")
            .add_translation("zho", "您确定要删除此保存的游戏吗？"),
        
        Translation::new(TranslationKey::ConfirmDelete)
            .add_translation("eng", "Yes, Delete")
            .add_translation("spa", "Sí, Eliminar")
            .add_translation("fra", "Oui, Supprimer")
            .add_translation("deu", "Ja, Löschen")
            .add_translation("ita", "Sì, Elimina")
            .add_translation("por", "Sim, Excluir")
            .add_translation("rus", "Да, Удалить")
            .add_translation("jpn", "はい、削除")
            .add_translation("kor", "네, 삭제")
            .add_translation("zho", "是的，删除"),
        
        Translation::new(TranslationKey::Cancel)
            .add_translation("eng", "Cancel")
            .add_translation("spa", "Cancelar")
            .add_translation("fra", "Annuler")
            .add_translation("deu", "Abbrechen")
            .add_translation("ita", "Annulla")
            .add_translation("por", "Cancelar")
            .add_translation("rus", "Отмена")
            .add_translation("jpn", "キャンセル")
            .add_translation("kor", "취소")
            .add_translation("zho", "取消"),
        
        // Settings UI
        Translation::new(TranslationKey::GameCrisesFolder)
            .add_translation("eng", "Crises Folder:")
            .add_translation("spa", "Carpeta de Crisis:")
            .add_translation("fra", "Dossier de Crises:")
            .add_translation("deu", "Krisen-Ordner:")
            .add_translation("ita", "Cartella Crisi:")
            .add_translation("por", "Pasta de Crises:")
            .add_translation("rus", "Папка Кризисов:")
            .add_translation("jpn", "クライシスフォルダ:")
            .add_translation("kor", "위기 폴더:")
            .add_translation("zho", "危机文件夹:"),
        
        Translation::new(TranslationKey::EnterCrisesFolderPath)
            .add_translation("eng", "Enter crises folder path...")
            .add_translation("spa", "Ingresa ruta de carpeta de crisis...")
            .add_translation("fra", "Entrez le chemin du dossier de crises...")
            .add_translation("deu", "Krisen-Ordnerpfad eingeben...")
            .add_translation("ita", "Inserisci percorso cartella crisi...")
            .add_translation("por", "Digite o caminho da pasta de crises...")
            .add_translation("rus", "Введите путь к папке кризисов...")
            .add_translation("jpn", "クライシスフォルダパスを入力...")
            .add_translation("kor", "위기 폴더 경로 입력...")
            .add_translation("zho", "输入危机文件夹路径..."),
        
        Translation::new(TranslationKey::CrisesFolderExplanation)
            .add_translation("eng", "This folder will hold additional Crisis games which can be played.")
            .add_translation("spa", "Esta carpeta contendrá juegos de Crisis adicionales que se pueden jugar.")
            .add_translation("fra", "Ce dossier contiendra des jeux de Crise supplémentaires qui peuvent être joués.")
            .add_translation("deu", "Dieser Ordner wird zusätzliche Krisen-Spiele enthalten, die gespielt werden können.")
            .add_translation("ita", "Questa cartella conterrà giochi di Crisi aggiuntivi che possono essere giocati.")
            .add_translation("por", "Esta pasta conterá jogos de Crise adicionais que podem ser jogados.")
            .add_translation("rus", "Эта папка будет содержать дополнительные игры Кризисов, в которые можно играть.")
            .add_translation("jpn", "このフォルダには、プレイできる追加のクライシスゲームが格納されます。")
            .add_translation("kor", "이 폴더에는 플레이할 수 있는 추가 위기 게임이 저장됩니다.")
            .add_translation("zho", "此文件夹将包含可以玩的额外危机游戏。"),
        
        Translation::new(TranslationKey::OpenFolder)
            .add_translation("eng", "Open")
            .add_translation("spa", "Abrir")
            .add_translation("fra", "Ouvrir")
            .add_translation("deu", "Öffnen")
            .add_translation("ita", "Apri")
            .add_translation("por", "Abrir")
            .add_translation("rus", "Открыть")
            .add_translation("jpn", "開く")
            .add_translation("kor", "열기")
            .add_translation("zho", "打开"),
        
        Translation::new(TranslationKey::SettingsStoragePath)
            .add_translation("eng", "Settings Storage Path:")
            .add_translation("spa", "Ruta de Almacenamiento de Configuración:")
            .add_translation("fra", "Chemin de Stockage des Paramètres:")
            .add_translation("deu", "Einstellungs-Speicherpfad:")
            .add_translation("ita", "Percorso di Archiviazione Impostazioni:")
            .add_translation("por", "Caminho de Armazenamento de Configurações:")
            .add_translation("rus", "Путь Хранения Настроек:")
            .add_translation("jpn", "設定保存パス:")
            .add_translation("kor", "설정 저장 경로:")
            .add_translation("zho", "设置存储路径:"),
        
        Translation::new(TranslationKey::SettingsStorageExplanation)
            .add_translation("eng", "This is where your game settings and save files are stored.")
            .add_translation("spa", "Aquí es donde se almacenan la configuración del juego y los archivos guardados.")
            .add_translation("fra", "C'est ici que vos paramètres de jeu et fichiers de sauvegarde sont stockés.")
            .add_translation("deu", "Hier werden Ihre Spieleinstellungen und Speicherdateien gespeichert.")
            .add_translation("ita", "Qui vengono archiviate le impostazioni del gioco e i file di salvataggio.")
            .add_translation("por", "Aqui são armazenadas as configurações do jogo e arquivos de salvamento.")
            .add_translation("rus", "Здесь хранятся настройки игры и файлы сохранений.")
            .add_translation("jpn", "ここにゲーム設定とセーブファイルが保存されます。")
            .add_translation("kor", "여기에 게임 설정과 저장 파일이 저장됩니다.")
            .add_translation("zho", "这里存储着您的游戏设置和保存文件。"),
        
        Translation::new(TranslationKey::DifficultyLevel)
            .add_translation("eng", "Difficulty Level:")
            .add_translation("spa", "Nivel de Dificultad:")
            .add_translation("fra", "Niveau de Difficulté:")
            .add_translation("deu", "Schwierigkeitsgrad:")
            .add_translation("ita", "Livello di Difficoltà:")
            .add_translation("por", "Nível de Dificuldade:")
            .add_translation("rus", "Уровень Сложности:")
            .add_translation("jpn", "難易度レベル:")
            .add_translation("kor", "난이도:")
            .add_translation("zho", "难度级别:"),
        
        Translation::new(TranslationKey::SelectDifficulty)
            .add_translation("eng", "Select difficulty")
            .add_translation("spa", "Seleccionar dificultad")
            .add_translation("fra", "Sélectionner la difficulté")
            .add_translation("deu", "Schwierigkeit auswählen")
            .add_translation("ita", "Seleziona difficoltà")
            .add_translation("por", "Selecionar dificuldade")
            .add_translation("rus", "Выберите сложность")
            .add_translation("jpn", "難易度を選択")
            .add_translation("kor", "난이도 선택")
            .add_translation("zho", "选择难度"),
        
        Translation::new(TranslationKey::Autosave)
            .add_translation("eng", "Autosave:")
            .add_translation("spa", "Guardado Automático:")
            .add_translation("fra", "Sauvegarde Auto:")
            .add_translation("deu", "Automatisches Speichern:")
            .add_translation("ita", "Salvataggio Automatico:")
            .add_translation("por", "Salvamento Automático:")
            .add_translation("rus", "Автосохранение:")
            .add_translation("jpn", "自動保存:")
            .add_translation("kor", "자동 저장:")
            .add_translation("zho", "自动保存:"),
        
        Translation::new(TranslationKey::Language)
            .add_translation("eng", "Language:")
            .add_translation("spa", "Idioma:")
            .add_translation("fra", "Langue:")
            .add_translation("deu", "Sprache:")
            .add_translation("ita", "Lingua:")
            .add_translation("por", "Idioma:")
            .add_translation("rus", "Язык:")
            .add_translation("jpn", "言語:")
            .add_translation("kor", "언어:")
            .add_translation("zho", "语言:"),
        
        Translation::new(TranslationKey::SelectLanguage)
            .add_translation("eng", "Select language")
            .add_translation("spa", "Seleccionar idioma")
            .add_translation("fra", "Sélectionner la langue")
            .add_translation("deu", "Sprache auswählen")
            .add_translation("ita", "Seleziona lingua")
            .add_translation("por", "Selecionar idioma")
            .add_translation("rus", "Выберите язык")
            .add_translation("jpn", "言語を選択")
            .add_translation("kor", "언어 선택")
            .add_translation("zho", "选择语言"),
        
        Translation::new(TranslationKey::FontScale)
            .add_translation("eng", "Font Scale:")
            .add_translation("spa", "Escala de Fuente:")
            .add_translation("fra", "Échelle de Police:")
            .add_translation("deu", "Schriftgröße:")
            .add_translation("ita", "Scala Font:")
            .add_translation("por", "Escala da Fonte:")
            .add_translation("rus", "Масштаб Шрифта:")
            .add_translation("jpn", "フォントスケール:")
            .add_translation("kor", "글꼴 크기:")
            .add_translation("zho", "字体大小:"),
        
        Translation::new(TranslationKey::FontScaleExplanation)
            .add_translation("eng", "Adjust the size of text throughout the application (0.1x - 2.0x)")
            .add_translation("spa", "Ajustar el tamaño del texto en toda la aplicación (0.1x - 2.0x)")
            .add_translation("fra", "Ajuster la taille du texte dans toute l'application (0.1x - 2.0x)")
            .add_translation("deu", "Textgröße in der gesamten Anwendung anpassen (0.1x - 2.0x)")
            .add_translation("ita", "Regola la dimensione del testo in tutta l'applicazione (0.1x - 2.0x)")
            .add_translation("por", "Ajustar o tamanho do texto em toda a aplicação (0.1x - 2.0x)")
            .add_translation("rus", "Настройка размера текста во всем приложении (0.1x - 2.0x)")
            .add_translation("jpn", "アプリケーション全体のテキストサイズを調整 (0.1x - 2.0x)")
            .add_translation("kor", "애플리케이션 전체 텍스트 크기 조정 (0.1x - 2.0x)")
            .add_translation("zho", "调整整个应用程序的文本大小 (0.1x - 2.0x)"),
        
        // Game Interface
        Translation::new(TranslationKey::WhatDoYouChoose)
            .add_translation("eng", "What do you choose?")
            .add_translation("spa", "¿Qué eliges?")
            .add_translation("fra", "Que choisissez-vous?")
            .add_translation("deu", "Was wählen Sie?")
            .add_translation("ita", "Cosa scegli?")
            .add_translation("por", "O que você escolhe?")
            .add_translation("rus", "Что вы выбираете?")
            .add_translation("jpn", "何を選びますか？")
            .add_translation("kor", "무엇을 선택하시겠습니까?")
            .add_translation("zho", "你选择什么？"),
        
        Translation::new(TranslationKey::PlayingAs)
            .add_translation("eng", "Playing as: {character_name}")
            .add_translation("spa", "Jugando como: {character_name}")
            .add_translation("fra", "Jouant en tant que: {character_name}")
            .add_translation("deu", "Spielen als: {character_name}")
            .add_translation("ita", "Giocando come: {character_name}")
            .add_translation("por", "Jogando como: {character_name}")
            .add_translation("rus", "Играя как: {character_name}")
            .add_translation("jpn", "{character_name}としてプレイ")
            .add_translation("kor", "{character_name}로 플레이")
            .add_translation("zho", "扮演: {character_name}"),
        
        Translation::new(TranslationKey::LoadingCrisis)
            .add_translation("eng", "Loading crisis...")
            .add_translation("spa", "Cargando crisis...")
            .add_translation("fra", "Chargement de la crise...")
            .add_translation("deu", "Krise wird geladen...")
            .add_translation("ita", "Caricamento crisi...")
            .add_translation("por", "Carregando crise...")
            .add_translation("rus", "Загрузка кризиса...")
            .add_translation("jpn", "クライシスを読み込み中...")
            .add_translation("kor", "위기 상황 로딩 중...")
            .add_translation("zho", "加载危机中..."),
        
        Translation::new(TranslationKey::ReturnToMenu)
            .add_translation("eng", "Return to Menu")
            .add_translation("spa", "Volver al Menú")
            .add_translation("fra", "Retour au Menu")
            .add_translation("deu", "Zurück zum Menü")
            .add_translation("ita", "Torna al Menu")
            .add_translation("por", "Voltar ao Menu")
            .add_translation("rus", "Вернуться в Меню")
            .add_translation("jpn", "メニューに戻る")
            .add_translation("kor", "메뉴로 돌아가기")
            .add_translation("zho", "返回菜单"),
        
        Translation::new(TranslationKey::SaveAndQuit)
            .add_translation("eng", "Save & Quit")
            .add_translation("spa", "Guardar y Salir")
            .add_translation("fra", "Sauvegarder et Quitter")
            .add_translation("deu", "Speichern & Beenden")
            .add_translation("ita", "Salva ed Esci")
            .add_translation("por", "Salvar e Sair")
            .add_translation("rus", "Сохранить и Выйти")
            .add_translation("jpn", "保存して終了")
            .add_translation("kor", "저장하고 나가기")
            .add_translation("zho", "保存并退出"),
        
        Translation::new(TranslationKey::Quit)
            .add_translation("eng", "Quit")
            .add_translation("spa", "Salir")
            .add_translation("fra", "Quitter")
            .add_translation("deu", "Beenden")
            .add_translation("ita", "Esci")
            .add_translation("por", "Sair")
            .add_translation("rus", "Выйти")
            .add_translation("jpn", "終了")
            .add_translation("kor", "나가기")
            .add_translation("zho", "退出"),
        
        Translation::new(TranslationKey::End)
            .add_translation("eng", "--- END ---")
            .add_translation("spa", "--- FIN ---")
            .add_translation("fra", "--- FIN ---")
            .add_translation("deu", "--- ENDE ---")
            .add_translation("ita", "--- FINE ---")
            .add_translation("por", "--- FIM ---")
            .add_translation("rus", "--- КОНЕЦ ---")
            .add_translation("jpn", "--- 終了 ---")
            .add_translation("kor", "--- 끝 ---")
            .add_translation("zho", "--- 结束 ---"),
        
        Translation::new(TranslationKey::SceneNotFound)
            .add_translation("eng", "Scene not found!")
            .add_translation("spa", "¡Escena no encontrada!")
            .add_translation("fra", "Scène introuvable!")
            .add_translation("deu", "Szene nicht gefunden!")
            .add_translation("ita", "Scena non trovata!")
            .add_translation("por", "Cena não encontrada!")
            .add_translation("rus", "Сцена не найдена!")
            .add_translation("jpn", "シーンが見つかりません！")
            .add_translation("kor", "장면을 찾을 수 없습니다!")
            .add_translation("zho", "未找到场景！"),
        
        Translation::new(TranslationKey::RequirementsNotMet)
            .add_translation("eng", "(Requirements not met)")
            .add_translation("spa", "(Requisitos no cumplidos)")
            .add_translation("fra", "(Exigences non remplies)")
            .add_translation("deu", "(Anforderungen nicht erfüllt)")
            .add_translation("ita", "(Requisiti non soddisfatti)")
            .add_translation("por", "(Requisitos não atendidos)")
            .add_translation("rus", "(Требования не выполнены)")
            .add_translation("jpn", "(要件が満たされていません)")
            .add_translation("kor", "(요구사항이 충족되지 않음)")
            .add_translation("zho", "(未满足要求)"),
        
        // Difficulty Levels
        Translation::new(TranslationKey::Easy)
            .add_translation("eng", "Easy")
            .add_translation("spa", "Fácil")
            .add_translation("fra", "Facile")
            .add_translation("deu", "Einfach")
            .add_translation("ita", "Facile")
            .add_translation("por", "Fácil")
            .add_translation("rus", "Легкий")
            .add_translation("jpn", "簡単")
            .add_translation("kor", "쉬움")
            .add_translation("zho", "简单"),
        
        Translation::new(TranslationKey::Medium)
            .add_translation("eng", "Medium")
            .add_translation("spa", "Medio")
            .add_translation("fra", "Moyen")
            .add_translation("deu", "Mittel")
            .add_translation("ita", "Medio")
            .add_translation("por", "Médio")
            .add_translation("rus", "Средний")
            .add_translation("jpn", "普通")
            .add_translation("kor", "보통")
            .add_translation("zho", "中等"),
        
        Translation::new(TranslationKey::Hard)
            .add_translation("eng", "Hard")
            .add_translation("spa", "Difícil")
            .add_translation("fra", "Difficile")
            .add_translation("deu", "Schwer")
            .add_translation("ita", "Difficile")
            .add_translation("por", "Difícil")
            .add_translation("rus", "Сложный")
            .add_translation("jpn", "難しい")
            .add_translation("kor", "어려움")
            .add_translation("zho", "困难"),
        
        // Generic
        Translation::new(TranslationKey::SelectFromLeftMenu)
            .add_translation("eng", "Select from left menu")
            .add_translation("spa", "Seleccionar del menú izquierdo")
            .add_translation("fra", "Sélectionner dans le menu de gauche")
            .add_translation("deu", "Aus dem linken Menü auswählen")
            .add_translation("ita", "Seleziona dal menu di sinistra")
            .add_translation("por", "Selecionar do menu esquerdo")
            .add_translation("rus", "Выберите из левого меню")
            .add_translation("jpn", "左メニューから選択")
            .add_translation("kor", "왼쪽 메뉴에서 선택")
            .add_translation("zho", "从左侧菜单选择"),
    ]
}
