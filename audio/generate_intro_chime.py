#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "numpy",
#     "scipy",
#     "soundfile",
# ]
# ///

"""
Generate various audio elements with string notation support.
Creates drum beats, guitar riffs, piano music from string patterns.
"""

import os
import sys
import numpy as np
import soundfile as sf
from scipy import signal
from pathlib import Path
import re

# Note frequency mapping (Equal temperament, A4 = 440Hz)
NOTE_FREQUENCIES = {
    'C0': 16.35, 'C#0': 17.32, 'Db0': 17.32, 'D0': 18.35, 'D#0': 19.45, 'Eb0': 19.45,
    'E0': 20.60, 'F0': 21.83, 'F#0': 23.12, 'Gb0': 23.12, 'G0': 24.50, 'G#0': 25.96, 'Ab0': 25.96,
    'A0': 27.50, 'A#0': 29.14, 'Bb0': 29.14, 'B0': 30.87,
    'C1': 32.70, 'C#1': 34.65, 'Db1': 34.65, 'D1': 36.71, 'D#1': 38.89, 'Eb1': 38.89,
    'E1': 41.20, 'F1': 43.65, 'F#1': 46.25, 'Gb1': 46.25, 'G1': 49.00, 'G#1': 51.91, 'Ab1': 51.91,
    'A1': 55.00, 'A#1': 58.27, 'Bb1': 58.27, 'B1': 61.74,
    'C2': 65.41, 'C#2': 69.30, 'Db2': 69.30, 'D2': 73.42, 'D#2': 77.78, 'Eb2': 77.78,
    'E2': 82.41, 'F2': 87.31, 'F#2': 92.50, 'Gb2': 92.50, 'G2': 98.00, 'G#2': 103.83, 'Ab2': 103.83,
    'A2': 110.00, 'A#2': 116.54, 'Bb2': 116.54, 'B2': 123.47,
    'C3': 130.81, 'C#3': 138.59, 'Db3': 138.59, 'D3': 146.83, 'D#3': 155.56, 'Eb3': 155.56,
    'E3': 164.81, 'F3': 174.61, 'F#3': 185.00, 'Gb3': 185.00, 'G3': 196.00, 'G#3': 207.65, 'Ab3': 207.65,
    'A3': 220.00, 'A#3': 233.08, 'Bb3': 233.08, 'B3': 246.94,
    'C4': 261.63, 'C#4': 277.18, 'Db4': 277.18, 'D4': 293.66, 'D#4': 311.13, 'Eb4': 311.13,
    'E4': 329.63, 'F4': 349.23, 'F#4': 369.99, 'Gb4': 369.99, 'G4': 392.00, 'G#4': 415.30, 'Ab4': 415.30,
    'A4': 440.00, 'A#4': 466.16, 'Bb4': 466.16, 'B4': 493.88,
    'C5': 523.25, 'C#5': 554.37, 'Db5': 554.37, 'D5': 587.33, 'D#5': 622.25, 'Eb5': 622.25,
    'E5': 659.25, 'F5': 698.46, 'F#5': 739.99, 'Gb5': 739.99, 'G5': 783.99, 'G#5': 830.61, 'Ab5': 830.61,
    'A5': 880.00, 'A#5': 932.33, 'Bb5': 932.33, 'B5': 987.77,
    'C6': 1046.50, 'C#6': 1108.73, 'Db6': 1108.73, 'D6': 1174.66, 'D#6': 1244.51, 'Eb6': 1244.51,
    'E6': 1318.51, 'F6': 1396.91, 'F#6': 1479.98, 'Gb6': 1479.98, 'G6': 1567.98, 'G#6': 1661.22, 'Ab6': 1661.22,
    'A6': 1760.00, 'A#6': 1864.66, 'Bb6': 1864.66, 'B6': 1975.53,
}

def create_envelope(duration, sample_rate, attack=0.01, decay=0.1, sustain=0.7, release=0.5):
    """Create an ADSR envelope for the sound."""
    total_samples = int(duration * sample_rate)
    attack_samples = int(attack * sample_rate)
    decay_samples = int(decay * sample_rate)
    release_samples = int(release * sample_rate)
    sustain_samples = total_samples - attack_samples - decay_samples - release_samples
    
    if sustain_samples < 0:
        sustain_samples = 0
        release_samples = total_samples - attack_samples - decay_samples
    
    envelope = np.zeros(total_samples)
    
    if attack_samples > 0:
        envelope[:attack_samples] = np.linspace(0, 1, attack_samples)
    
    start = attack_samples
    if decay_samples > 0:
        end = start + decay_samples
        envelope[start:end] = np.linspace(1, sustain, decay_samples)
        start = end
    
    if sustain_samples > 0:
        end = start + sustain_samples
        envelope[start:end] = sustain
        start = end
    
    if len(envelope[start:]) > 0:
        envelope[start:] = np.linspace(sustain, 0, len(envelope[start:]))
    
    return envelope

def parse_piano_notation(notation):
    """
    Parse piano notation string into notes.
    Format: "C4/q D4/q E4/h F4/q G4/w" 
    Note format: NOTE/DURATION where:
    - NOTE: C, D, E, F, G, A, B with optional # or b and octave number
    - DURATION: w=whole, h=half, q=quarter, e=eighth, s=sixteenth, .=rest
    """
    notes = []
    for token in notation.split():
        if '/' in token:
            note, duration = token.split('/')
            if note == '.':  # Rest
                notes.append((None, duration))
            else:
                notes.append((note.upper(), duration))
        else:
            # Default to quarter note if no duration specified
            notes.append((token.upper(), 'q'))
    
    return notes

def parse_drum_notation(pattern):
    """
    Parse drum pattern string.
    Format: "K.S.H.S." where:
    - K = kick drum
    - S = snare drum  
    - H = hi-hat (closed)
    - O = hi-hat (open)
    - . = rest
    Each character represents a 16th note
    """
    return list(pattern.upper())

def parse_guitar_notation(notation):
    """
    Parse guitar tablature notation.
    Format: "E2-1 A2-1 D3-1 G3-1" where:
    - First part is note name and octave
    - Number after dash is duration in beats
    """
    notes = []
    for token in notation.split():
        if '-' in token:
            note, duration = token.split('-')
            notes.append((note.upper(), float(duration)))
        else:
            notes.append((token.upper(), 1.0))  # Default 1 beat
    
    return notes

def generate_kick_drum(sample_rate=44100, duration=0.5):
    """Generate a kick drum sound using synthesis."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    pitch_envelope = np.exp(-35 * t)
    body = np.sin(2 * np.pi * (60 * pitch_envelope + 40) * t)
    
    click_duration = 0.005
    click_samples = int(sample_rate * click_duration)
    click = np.random.normal(0, 0.1, len(t))
    click[click_samples:] = 0
    
    kick = body * 0.9 + click
    envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.01, sustain=0.3, release=0.2)
    kick *= envelope
    
    return kick

def generate_snare_drum(sample_rate=44100, duration=0.2):
    """Generate a snare drum sound using synthesis."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    tone = np.sin(2 * np.pi * 200 * t) * 0.5
    tone += np.sin(2 * np.pi * 250 * t) * 0.3
    
    noise = np.random.normal(0, 0.2, len(t))
    sos = signal.butter(4, 2000, 'hp', fs=sample_rate, output='sos')
    noise = signal.sosfilt(sos, noise)
    
    snare = tone * 0.4 + noise * 0.6
    envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.02, sustain=0.1, release=0.15)
    snare *= envelope
    
    return snare

def generate_hihat(sample_rate=44100, duration=0.1, closed=True):
    """Generate a hi-hat sound using synthesis."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    hihat = np.random.normal(0, 0.1, len(t))
    sos = signal.butter(4, 8000, 'hp', fs=sample_rate, output='sos')
    hihat = signal.sosfilt(sos, hihat)
    
    if closed:
        envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.01, sustain=0.0, release=0.08)
    else:
        envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.02, sustain=0.2, release=0.3)
    
    hihat *= envelope
    return hihat

def karplus_strong(frequency, duration, sample_rate):
    """Karplus-Strong string synthesis algorithm."""
    samples = int(sample_rate * duration)
    delay_samples = int(sample_rate / frequency)
    
    noise = np.random.normal(0, 1, delay_samples)
    output = np.zeros(samples)
    output[:delay_samples] = noise
    
    for i in range(delay_samples, samples):
        output[i] = 0.996 * 0.5 * (output[i - delay_samples] + output[i - delay_samples - 1])
    
    return output

def piano_note(frequency, duration, sample_rate):
    """Generate a piano note with harmonics."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    harmonics = [
        (1.0, 1.0), (2.0, 0.35), (3.0, 0.20), (4.0, 0.15), (5.0, 0.10), (6.0, 0.08)
    ]
    
    note = np.zeros_like(t)
    for harmonic, amplitude in harmonics:
        decay = np.exp(-2.5 * harmonic * t)
        note += amplitude * np.sin(2 * np.pi * frequency * harmonic * t) * decay
    
    envelope = create_envelope(duration, sample_rate, attack=0.002, decay=0.1, sustain=0.3, release=0.5)
    note *= envelope
    
    return note

def generate_piano_from_notation(notation, sample_rate=44100, bpm=120):
    """Generate piano from notation string."""
    notes = parse_piano_notation(notation)
    
    # Duration mapping (in beats)
    duration_map = {'w': 4, 'h': 2, 'q': 1, 'e': 0.5, 's': 0.25}
    beat_duration = 60.0 / bpm
    
    piano_track = []
    
    for note, duration_symbol in notes:
        duration_beats = duration_map.get(duration_symbol, 1)
        duration_seconds = duration_beats * beat_duration
        
        if note is None:  # Rest
            silence = np.zeros(int(sample_rate * duration_seconds))
            piano_track.append(silence)
        else:
            frequency = NOTE_FREQUENCIES.get(note, 261.63)  # Default to C4
            note_audio = piano_note(frequency, duration_seconds, sample_rate)
            piano_track.append(note_audio)
    
    return np.concatenate(piano_track) if piano_track else np.array([])

def generate_drums_from_notation(pattern, sample_rate=44100, bpm=120):
    """Generate drums from pattern notation."""
    drum_hits = parse_drum_notation(pattern)
    
    sixteenth_duration = (60.0 / bpm) / 4  # 16th note duration
    
    # Generate drum samples
    kick = generate_kick_drum(sample_rate, duration=0.5)
    snare = generate_snare_drum(sample_rate, duration=0.2)
    hihat_closed = generate_hihat(sample_rate, duration=0.1, closed=True)
    hihat_open = generate_hihat(sample_rate, duration=0.2, closed=False)
    
    total_duration = len(drum_hits) * sixteenth_duration
    drums = np.zeros(int(sample_rate * total_duration))
    
    for i, hit in enumerate(drum_hits):
        position = int(i * sixteenth_duration * sample_rate)
        
        if hit == 'K' and position + len(kick) < len(drums):
            drums[position:position + len(kick)] += kick * 0.8
        elif hit == 'S' and position + len(snare) < len(drums):
            drums[position:position + len(snare)] += snare * 0.7
        elif hit == 'H' and position + len(hihat_closed) < len(drums):
            drums[position:position + len(hihat_closed)] += hihat_closed * 0.4
        elif hit == 'O' and position + len(hihat_open) < len(drums):
            drums[position:position + len(hihat_open)] += hihat_open * 0.4
    
    return drums

def generate_guitar_from_notation(notation, sample_rate=44100, bpm=120):
    """Generate guitar from notation string."""
    notes = parse_guitar_notation(notation)
    
    beat_duration = 60.0 / bpm
    guitar_track = []
    
    for note, duration_beats in notes:
        duration_seconds = duration_beats * beat_duration
        
        if note == '.' or note == 'REST':  # Rest
            silence = np.zeros(int(sample_rate * duration_seconds))
            guitar_track.append(silence)
        else:
            frequency = NOTE_FREQUENCIES.get(note, 82.41)  # Default to E2
            
            # Generate guitar note with Karplus-Strong
            root = karplus_strong(frequency, duration_seconds, sample_rate)
            fifth = karplus_strong(frequency * 1.5, duration_seconds, sample_rate)
            
            # Mix and add distortion
            chord = root + fifth * 0.7
            chord = np.tanh(chord * 1.5) * 0.6
            
            guitar_track.append(chord)
    
    return np.concatenate(guitar_track) if guitar_track else np.array([])

def create_30_second_composition(sample_rate=44100):
    """Create a complete 30-second musical composition."""
    bpm = 120
    
    # Piano melody (simplified notation)
    piano_melody = """
    C4/q E4/q G4/q C5/q E5/q G5/q C5/q G4/q
    A3/q C4/q E4/q A4/q C5/q E5/q A4/q E4/q
    F3/q A3/q C4/q F4/q A4/q C5/q F4/q C4/q
    G3/q B3/q D4/q G4/q B4/q D5/q G4/q D4/q
    C4/q E4/q G4/e C5/e E5/q G5/h
    """
    
    # Drum pattern (16 bars of 16th notes)
    drum_pattern = (
        "K.S.H.S.K.S.H.S." +  # Bar 1
        "K.S.H.S.K.K.H.S." +  # Bar 2 (with kick variation)
        "K.S.H.S.K.S.H.S." +  # Bar 3
        "K.S.H.S.K.S.O.S."    # Bar 4 (with open hihat)
    ) * 4  # Repeat 4 times for full composition
    
    # Guitar progression (power chords)
    guitar_riff = """
    E2-2 G2-2 A2-2 C3-2
    E2-1 E2-1 G2-1 G2-1 A2-1 A2-1 C3-2
    E2-4 G2-4 A2-4 C3-4
    """
    
    print("Generating piano melody...")
    piano = generate_piano_from_notation(piano_melody, sample_rate, bpm)
    
    print("Generating drum pattern...")
    drums = generate_drums_from_notation(drum_pattern, sample_rate, bpm)
    
    print("Generating guitar riff...")
    #guitar = generate_guitar_from_notation(guitar_riff, sample_rate, bpm)
    
    # Electronic bass line (simple pattern)
    def generate_bass_line(sample_rate, duration, bpm):
        """Generate a simple electronic bass line."""
        t = np.linspace(0, duration, int(sample_rate * duration))
        bass_notes = [65.41, 82.41, 87.31, 98.00]  # C2, E2, F2, G2
        
        bass = np.zeros_like(t)
        note_duration = 60.0 / bpm  # Quarter notes
        
        for i, freq in enumerate(bass_notes * 8):  # Repeat pattern
            start_time = i * note_duration
            if start_time >= duration:
                break
                
            start_idx = int(start_time * sample_rate)
            end_idx = min(start_idx + int(note_duration * sample_rate), len(t))
            
            if start_idx < len(t):
                t_slice = t[start_idx:end_idx] - t[start_idx]
                # Square wave bass
                wave = np.sign(np.sin(2 * np.pi * freq * t_slice))
                envelope = create_envelope(note_duration, sample_rate, 
                                         attack=0.001, decay=0.05, sustain=0.7, release=0.2)
                if len(envelope) > len(wave):
                    envelope = envelope[:len(wave)]
                bass[start_idx:start_idx + len(wave)] += wave * envelope[:len(wave)] * 0.3
        
        return bass
    
    print("Generating bass line...")
    #bass = generate_bass_line(sample_rate, 30, bpm)
    
    # Ensure all tracks are same length (30 seconds)
    target_length = int(30 * sample_rate)
    
    def pad_or_truncate(audio, target_length):
        if len(audio) < target_length:
            return np.pad(audio, (0, target_length - len(audio)))
        else:
            return audio[:target_length]
    
    piano = pad_or_truncate(piano, target_length)
    drums = pad_or_truncate(drums, target_length)
    #guitar = pad_or_truncate(guitar, target_length)
    #bass = pad_or_truncate(bass, target_length)
    
    # Mix all tracks
    print("Mixing all tracks...")
    mix = (drums * 0.7 +      # Drums prominent
           #bass * 0.6 +       # Bass foundation
           #guitar * 0.5 +     # Guitar mid-level
           piano * 0.8)       # Piano melody prominent
    
    # Apply compression
    threshold = 0.8
    ratio = 3
    compressed = np.where(
        np.abs(mix) > threshold,
        np.sign(mix) * (threshold + (np.abs(mix) - threshold) / ratio),
        mix
    )
    
    # Final limiting and normalization
    final_mix = np.tanh(compressed * 0.9) * 0.95
    final_mix = final_mix / np.max(np.abs(final_mix)) * 0.9
    
    # Umm... Lower absolute volume to 30%?
    final_mix = 0.30 * final_mix

    return final_mix

def main():
    """Generate and save the audio composition."""
    print("Advanced Audio Generator - String Notation Support")
    print("=" * 55)
    
    sample_rate = 44100
    
    # Get output path from command line or use default
    output_path = '/tmp/composition.wav' if len(sys.argv) < 2 else sys.argv[1]
    
    # Ensure output directory exists
    os.makedirs(os.path.dirname(output_path) or '.', exist_ok=True)
    
    # Generate the 30-second composition
    print("\nGenerating 30-second composition...")
    composition = create_30_second_composition(sample_rate)
    
    # Save the composition
    sf.write(output_path, composition, sample_rate)
    print(f"Saved composition: {output_path}")


if __name__ == "__main__":
    main()
