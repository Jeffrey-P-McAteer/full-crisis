
export function os_prefers_dark() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function browser_go_back() {
    window.history.back();
}

// Global audio context for background audio playback
const context = new AudioContext();
let currentAudioSource = null;

// Play background audio from a Vec<u8> (Uint8Array)
export function play_background_audio(bytes) {
    try {
        // Stop any currently playing audio
        if (currentAudioSource) {
            currentAudioSource.stop();
            currentAudioSource = null;
        }
        
        // If empty array, just stop audio and return
        if (!bytes || bytes.length === 0) {
            return;
        }
        
        // Create buffer and copy the bytes
        const buffer = context.createBuffer(2, bytes.length, context.sampleRate);
        const arrayBuffer = new ArrayBuffer(bytes.length);
        const view = new Uint8Array(arrayBuffer);
        view.set(new Uint8Array(bytes), 0);
        
        // Decode the audio data
        context.decodeAudioData(arrayBuffer.slice(0), function(decodedBuffer) {
            // Create and configure the audio source
            const source = context.createBufferSource();
            source.buffer = decodedBuffer;
            source.loop = true; // Loop the audio
            source.connect(context.destination);
            
            // Play the audio
            source.start();
            currentAudioSource = source;
        }, function(error) {
            console.error('Error decoding audio data:', error);
        });
    } catch (error) {
        console.error('Error in play_background_audio:', error);
    }
}

// Stop any currently playing background audio
export function stop_background_audio() {
    if (currentAudioSource) {
        currentAudioSource.stop();
        currentAudioSource = null;
    }
}
