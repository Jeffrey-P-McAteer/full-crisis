use serde::{Serialize, Deserialize};

/// Central translation keys for the GUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TranslationKey {
    // Main Menu
    ContinueGame,
    NewGame,
    Settings,
    Licenses,
    QuitGame,
    
    // New Game UI
    PlayerName,
    EnterName,
    GameType,
    SelectGameType,
    Go,
    
    // Continue Game UI
    SavedGame,
    SelectGame,
    Play,
    Delete,
    DeleteGame,
    ConfirmDelete,
    Cancel,
    
    // Settings UI
    GameCrisesFolder,
    EnterCrisesFolderPath,
    CrisesFolderExplanation,
    OpenFolder,
    SettingsStoragePath,
    SettingsStorageExplanation,
    DifficultyLevel,
    SelectDifficulty,
    Autosave,
    Language,
    SelectLanguage,
    FontScale,
    FontScaleExplanation,
    
    // Game Interface
    WhatDoYouChoose,
    PlayingAs,
    LoadingCrisis,
    ReturnToMenu,
    End,
    SceneNotFound,
    RequirementsNotMet,
    SaveAndQuit,
    Quit,
    
    // Difficulty Levels
    Easy,
    Medium,
    Hard,
    
    // Generic
    SelectFromLeftMenu,
}