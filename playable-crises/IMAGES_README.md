# Image Support for Playable Crises

The crisis game now supports background images and speaking character images for enhanced visual storytelling.

## Scene Image Configuration

In your crisis.toml file, you can add image paths to any scene:

```toml
[scenes.scene_name]
text.eng = "Your scene text here"
text.spa = "Tu texto de escena aquí"
background_image = "Crisis_Folder/background.png"
speaking_character_image = "Crisis_Folder/character.png"
```

## Image Requirements

- **Background Images**: Any PNG, JPG, or supported image format. Will be scaled to fill the entire screen background.
- **Speaking Character Images**: Any PNG, JPG, or supported image format. Will be displayed at 200x300 pixels in the lower-right corner.
- **File Paths**: Relative to the playable-crises folder. Use the format "Crisis_Folder_Name/image_file.png"

## Layout

The new game UI layout positions elements as follows:
- **Top Center**: Game title, character name, and variables
- **Center**: Story text in a styled container
- **Lower Left**: Player choice buttons (anchored to bottom-left)
- **Lower Right**: Speaking character image (anchored to bottom-right)
- **Background**: Full-screen background image behind all UI elements

## Translation Note

Unlike text content, image paths do NOT need to be translated. Use the same image paths for all languages, as images are visual and don't require localization (unless you specifically want different images per language).

## Example Usage

See the Fire_Dispatch crisis for examples of how to use background_image and speaking_character_image in scene definitions.