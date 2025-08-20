# Crisis Format Documentation

This directory contains all playable crisis scenarios for the Full Crisis game. Each crisis is a standalone interactive story with branching narratives, character dialogue, and decision-making mechanics.

## Directory Structure

Each crisis follows this structure:

```
playable-crises/
├── Crisis_Name/
│   ├── crisis.toml          # Main crisis configuration
│   ├── scenes/              # Individual scene files (optional)
│   │   ├── scene1.toml
│   │   ├── scene2.toml
│   │   └── ...
│   └── assets/              # Crisis-specific assets (images, audio)
│       ├── background1.png
│       ├── character1.png
│       └── ...
```

## Crisis Configuration (`crisis.toml`)

The main configuration file defines the crisis metadata, story flow, and mechanics.

### Basic Structure

```toml
[metadata]
id = "crisis_identifier"
version = "1.0"
author = "Author Name"
description_key = "description_identifier"

[name]
eng = "English Crisis Name"
spa = "Spanish Crisis Name"
# Add more languages as needed

[description]
eng = "English description of the crisis scenario"
spa = "Spanish description of the crisis scenario"

[character_names]
male_eng = ["John Smith", "Michael Johnson", "David Wilson"]
female_eng = ["Sarah Miller", "Jessica Brown", "Ashley Davis"]
male_spa = ["Carlos Mendoza", "Luis Jiménez", "Rafael Torres"]
female_spa = ["Patricia Vega", "Monica Delgado", "Claudia Sandoval"]
# Character names can also be role-specific:
# dispatcher_male_eng = ["Dispatcher Tom Wilson"]

[story]
starting_scene = "opening_scene"
default_language = "eng"

[mechanics]
time_limit_minutes = 15
save_progress = true
allow_restart = true
track_decisions = true

[conditions]
variables = ["resource_count", "stress_level", "reputation"]

[conditions.choice_effects]
"quick_action" = { stress_level = -2, resource_count = -1 }
"careful_planning" = { stress_level = 1, reputation = 2 }
```

### Inline Scenes (Legacy Format)

Scenes can be defined directly in `crisis.toml`:

```toml
[scenes.opening_scene]
background_image = "Crisis_Name/background.png"
speaking_character_image = "Crisis_Name/character.png"

[scenes.opening_scene.text]
eng = "Welcome to the crisis scenario. Make your choice."
spa = "Bienvenido al escenario de crisis. Haz tu elección."

[[scenes.opening_scene.choices]]
text.eng = "Take immediate action"
text.spa = "Tomar acción inmediata"
leads_to = "action_scene"

[[scenes.opening_scene.choices]]
text.eng = "Gather more information"
text.spa = "Reunir más información"
leads_to = "information_scene"
```

## Scene Files (`scenes/*.toml`)

**Recommended approach**: Individual scene files in the `scenes/` subdirectory for better organization and maintainability.

### Scene File Structure

```toml
# Background and character images
background_image = "Crisis_Name/scene_background.png"
speaking_character_image = "Crisis_Name/character.png"

# Scene text in multiple languages
[text]
eng = "Scene description in English with {character_name} placeholder."
spa = "Descripción de la escena en español con marcador {character_name}."

# Player choices
[[choices]]
text.eng = "First choice text"
text.spa = "Texto de la primera opción"
leads_to = "next_scene_id"
requires = { stress_level = 5 }  # Optional: variable requirements
character_type = "dispatcher"    # Optional: character type requirement

[[choices]]
text.eng = "Second choice text"
text.spa = "Texto de la segunda opción"
leads_to = "alternative_scene"

# Optional: Continue in subfolder
continue_in_subfolder = "subfolder_name"
```

### Animated Character Images

Characters can have animated images that cycle automatically:

```toml
# Single image (traditional)
speaking_character_image = "Crisis_Name/character.png"

# Animated sequence (cycles every 500ms)
speaking_character_image = [
    "Crisis_Name/character_frame1.png",
    "Crisis_Name/character_frame2.png",
    "Crisis_Name/character_frame3.png"
]
```

### Text Input Fields

Scenes can collect text input from players:

```toml
[[choices]]
text.eng = "Enter your name"
text.spa = "Ingresa tu nombre"
leads_to = "next_scene"

[choices.text_input]
variable_name = "player_name"
input_type = "Text"
placeholder.eng = "Your name here"
placeholder.spa = "Tu nombre aquí"
min_length = 2
max_length = 50
```

For numeric input:

```toml
[choices.text_input]
variable_name = "resource_amount"
input_type = "Number"
min_value = 1
max_value = 100
```

## Example Crisis: Fire_Dispatch

A complete example showing modern scene structure:

### `Fire_Dispatch/crisis.toml`

```toml
[metadata]
id = "fire_dispatch"
version = "1.0"
author = "Crisis Game System"
description_key = "fire_dispatch_desc"

[name]
eng = "Fire Dispatch"
spa = "Despacho de Emergencia"

[description]
eng = "You are an emergency dispatcher receiving urgent fire calls. Every decision affects response times and lives at stake."
spa = "Eres un despachador de emergencias recibiendo llamadas urgentes de incendio. Cada decisión afecta los tiempos de respuesta y las vidas en riesgo."

[character_names]
male_eng = ["Dispatcher Tom Wilson", "Dispatcher Mark Stevens"]
female_eng = ["Dispatcher Sarah Miller", "Dispatcher Jessica Brown"]
male_spa = ["Despachador Carlos Mendoza", "Despachador Luis Jiménez"]
female_spa = ["Despachadora Patricia Vega", "Despachadora Monica Delgado"]

[story]
starting_scene = "incoming_call"
default_language = "eng"

[mechanics]
time_limit_minutes = 10
save_progress = true
allow_restart = true
track_decisions = true

[conditions]
variables = ["available_trucks", "response_time", "caller_panic_level"]

[conditions.choice_effects]
"send_one_truck" = { available_trucks = -1 }
"send_full_response" = { available_trucks = -5 }
```

### `Fire_Dispatch/scenes/incoming_call.toml`

```toml
background_image = "Fire_Dispatch/background_from_firetruck_center.png"
speaking_character_image = [
    "Fire_Dispatch/panicked_caller_character.png",
    "Fire_Dispatch/panicked_caller_character_2.png"
]

[text]
eng = "{character_name}, you're working the night shift at Central Emergency Dispatch. It's 2:47 AM when your console lights up with an incoming 911 call. The caller sounds panicked: 'Help! There's a fire at my apartment building!'"
spa = "{character_name}, estás trabajando el turno nocturno en Despacho Central de Emergencias. Son las 2:47 AM cuando tu consola se ilumina con una llamada entrante al 911. La persona que llama suena en pánico: '¡Ayuda! ¡Hay un incendio en mi edificio!'"

[[choices]]
text.eng = "Ask for specific location and floor"
text.spa = "Preguntar por ubicación específica y piso"
leads_to = "gather_details"

[[choices]]
text.eng = "Immediately dispatch one fire truck"
text.spa = "Despachar inmediatamente un camión de bomberos"
leads_to = "quick_dispatch"

[[choices]]
text.eng = "Ask about number of people potentially trapped"
text.spa = "Preguntar sobre número de personas potencialmente atrapadas"
leads_to = "assess_casualties"
```

### `Fire_Dispatch/scenes/gather_details.toml`

```toml
background_image = "Fire_Dispatch/apartment_fire_background.png"
speaking_character_image = [
    "Fire_Dispatch/panicked_caller_character.png",
    "Fire_Dispatch/panicked_caller_character_2.png"
]

[text]
eng = "Caller: 'It's the Riverside Apartments on Oak Street! Third floor, but the smoke is getting thicker. I can see flames from the window!' You note this is a 6-story building with 48 units."
spa = "Llamador: '¡Son los Apartamentos Riverside en Oak Street! Tercer piso, ¡pero el humo se está espesando. ¡Puedo ver llamas desde la ventana!' Notas que es un edificio de 6 pisos con 48 unidades."

[[choices]]
text.eng = "Send standard response - 2 fire trucks"
text.spa = "Enviar respuesta estándar - 2 camiones de bomberos"
leads_to = "standard_response"

[[choices]]
text.eng = "Send enhanced response - 4 trucks plus rescue unit"
text.spa = "Enviar respuesta mejorada - 4 camiones más unidad de rescate"
leads_to = "enhanced_response"

[[choices]]
text.eng = "Send full alarm - 6 trucks, 2 rescue units, ambulances"
text.spa = "Enviar alarma completa - 6 camiones, 2 unidades de rescate, ambulancias"
leads_to = "full_alarm"
```

## Asset Organization

### Image Assets

- **Background images**: Scene backgrounds, locations, environments
- **Character images**: Speaking character portraits (static or animated frames)
- **UI elements**: Icons, buttons, overlays

### Naming Conventions

- Use descriptive names: `apartment_fire_background.png`, `panicked_caller_character.png`
- For animated characters: `character_frame1.png`, `character_frame2.png`, etc.
- Group by crisis: `Crisis_Name/asset_name.png`

## Testing and Validation

Use the built-in CLI test tool to validate crisis files:

```bash
# Test all crises
./full-crisis test

# Test with verbose output
./full-crisis test -v
```

The test tool validates:
- TOML file parsing
- Scene connectivity (no orphaned scenes)
- Asset references (missing images)
- Choice destinations (invalid scene references)
- Story flow integrity

## Language Support

The system supports multiple languages through localized text maps:

```toml
[text]
eng = "English text"
spa = "Spanish text"
fra = "French text"
```

Language fallback chain ensures graceful degradation when translations are missing.

## Best Practices

1. **Use scene files**: Prefer individual scene files over inline scenes for maintainability
2. **Descriptive IDs**: Use clear, descriptive scene and choice identifiers
3. **Asset optimization**: Compress images appropriately for the target platform
4. **Test thoroughly**: Use the validation tool frequently during development
5. **Consistent naming**: Follow established naming conventions for assets
6. **Balanced choices**: Provide meaningful choices that affect story outcomes
7. **Variable tracking**: Use the conditions system to create dynamic storylines
8. **Localization**: Support multiple languages from the start

## Migration from Legacy Format

To convert from inline scenes to scene files:

1. Extract each `[scenes.scene_name]` section to `scenes/scene_name.toml`
2. Move the scene content to the root level of the new file
3. Update asset references if needed
4. Test with `./full-crisis test` to ensure everything works
5. Remove the inline scenes from the main `crisis.toml`

The system supports both formats simultaneously, so migration can be gradual.