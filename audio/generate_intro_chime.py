#!/usr/bin/env python3
# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "numpy",
#     "scipy",
#     "soundfile",
# ]
# ///

"""
Generate various audio elements commonly found in audio software.
Creates drum beats, guitar riffs, piano, and electronic tones.
"""

import os
import sys
import numpy as np
import soundfile as sf
from scipy import signal
from pathlib import Path

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
    
    # Attack
    envelope[:attack_samples] = np.linspace(0, 1, attack_samples)
    
    # Decay
    start = attack_samples
    end = start + decay_samples
    envelope[start:end] = np.linspace(1, sustain, decay_samples)
    
    # Sustain
    start = end
    end = start + sustain_samples
    envelope[start:end] = sustain
    
    # Release
    start = end
    envelope[start:] = np.linspace(sustain, 0, len(envelope[start:]))
    
    return envelope

def generate_kick_drum(sample_rate=44100, duration=0.5):
    """Generate a kick drum sound using synthesis."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # Body: Low frequency sine wave with pitch envelope
    pitch_envelope = np.exp(-35 * t)
    body = np.sin(2 * np.pi * (60 * pitch_envelope + 40) * t)
    
    # Click: Short noise burst for attack
    click_duration = 0.005
    click_samples = int(sample_rate * click_duration)
    click = np.random.normal(0, 0.1, len(t))
    click[click_samples:] = 0
    
    # Combine and apply envelope
    kick = body * 0.9 + click
    envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.01, sustain=0.3, release=0.2)
    kick *= envelope
    
    return kick

def generate_snare_drum(sample_rate=44100, duration=0.2):
    """Generate a snare drum sound using synthesis."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # Tone component (200-250 Hz)
    tone = np.sin(2 * np.pi * 200 * t) * 0.5
    tone += np.sin(2 * np.pi * 250 * t) * 0.3
    
    # Noise component (snare rattle)
    noise = np.random.normal(0, 0.2, len(t))
    
    # High-pass filter the noise
    sos = signal.butter(4, 2000, 'hp', fs=sample_rate, output='sos')
    noise = signal.sosfilt(sos, noise)
    
    # Combine
    snare = tone * 0.4 + noise * 0.6
    envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.02, sustain=0.1, release=0.15)
    snare *= envelope
    
    return snare

def generate_hihat(sample_rate=44100, duration=0.1, closed=True):
    """Generate a hi-hat sound using synthesis."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # High frequency noise
    hihat = np.random.normal(0, 0.1, len(t))
    
    # High-pass filter
    sos = signal.butter(4, 8000, 'hp', fs=sample_rate, output='sos')
    hihat = signal.sosfilt(sos, hihat)
    
    # Envelope
    if closed:
        envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.01, sustain=0.0, release=0.08)
    else:
        envelope = create_envelope(duration, sample_rate, attack=0.001, decay=0.02, sustain=0.2, release=0.3)
    
    hihat *= envelope
    return hihat

def generate_drum_beat(sample_rate=44100, bpm=120, bars=2):
    """Generate a drum beat pattern."""
    beat_duration = 60.0 / bpm  # Duration of one beat in seconds
    bar_duration = beat_duration * 4  # 4/4 time signature
    total_duration = bar_duration * bars
    total_samples = int(total_duration * sample_rate)
    
    # Initialize drum track
    drums = np.zeros(total_samples)
    
    # Generate drum samples
    kick = generate_kick_drum(sample_rate)
    snare = generate_snare_drum(sample_rate)
    hihat = generate_hihat(sample_rate, closed=True)
    
    # Pattern (16th notes): K=kick, S=snare, H=hihat, .=rest
    # Basic rock pattern
    pattern = "K...H...S...H..."  # One bar pattern
    
    sixteenth_duration = beat_duration / 4
    
    for bar in range(bars):
        for i, hit in enumerate(pattern):
            position = int((bar * bar_duration + i * sixteenth_duration) * sample_rate)
            
            if hit == 'K' and position + len(kick) < len(drums):
                drums[position:position + len(kick)] += kick * 0.8
            elif hit == 'S' and position + len(snare) < len(drums):
                drums[position:position + len(snare)] += snare * 0.7
            elif hit == 'H' and position + len(hihat) < len(drums):
                drums[position:position + len(hihat)] += hihat * 0.4
    
    return drums

def generate_guitar_riff(sample_rate=44100, duration=4):
    """Generate a guitar riff using Karplus-Strong synthesis."""
    
    def karplus_strong(frequency, duration, sample_rate):
        """Karplus-Strong string synthesis algorithm."""
        samples = int(sample_rate * duration)
        delay_samples = int(sample_rate / frequency)
        
        # Initial noise burst (pluck)
        noise = np.random.normal(0, 1, delay_samples)
        
        # Initialize output
        output = np.zeros(samples)
        output[:delay_samples] = noise
        
        # Feedback loop with low-pass filter
        for i in range(delay_samples, samples):
            # Simple low-pass filter (averaging)
            output[i] = 0.996 * 0.5 * (output[i - delay_samples] + output[i - delay_samples - 1])
        
        return output
    
    # Power chord progression (E5, G5, A5, C5)
    riff_notes = [
        (82.41, 1.0),   # E2
        (98.00, 1.0),   # G2
        (110.00, 1.0),  # A2
        (130.81, 1.0),  # C3
    ]
    
    guitar = np.zeros(int(sample_rate * duration))
    position = 0
    
    for freq, note_duration in riff_notes:
        # Generate power chord (root + fifth)
        root = karplus_strong(freq, note_duration, sample_rate)
        fifth = karplus_strong(freq * 1.5, note_duration, sample_rate)
        octave = karplus_strong(freq * 2, note_duration, sample_rate)
        
        # Mix the notes
        chord = root + fifth * 0.7 + octave * 0.4
        
        # Add some distortion for rock sound
        chord = np.tanh(chord * 2) * 0.5
        
        # Place in the guitar track
        end_position = position + len(chord)
        if end_position <= len(guitar):
            guitar[position:end_position] += chord
        position = end_position
    
    return guitar

def generate_piano(sample_rate=44100, duration=4):
    """Generate piano sounds using additive synthesis."""
    
    def piano_note(frequency, duration, sample_rate):
        """Generate a piano note with harmonics."""
        t = np.linspace(0, duration, int(sample_rate * duration))
        
        # Fundamental and harmonics with piano-like amplitude ratios
        harmonics = [
            (1.0, 1.0),    # Fundamental
            (2.0, 0.35),   # 2nd harmonic
            (3.0, 0.20),   # 3rd harmonic
            (4.0, 0.15),   # 4th harmonic
            (5.0, 0.10),   # 5th harmonic
            (6.0, 0.08),   # 6th harmonic
        ]
        
        note = np.zeros_like(t)
        for harmonic, amplitude in harmonics:
            # Each harmonic has slightly different decay
            decay = np.exp(-2.5 * harmonic * t)
            note += amplitude * np.sin(2 * np.pi * frequency * harmonic * t) * decay
        
        # Piano-like envelope
        envelope = create_envelope(duration, sample_rate, 
                                 attack=0.002, decay=0.1, 
                                 sustain=0.3, release=0.5)
        note *= envelope
        
        return note
    
    # Chord progression (C major - Am - F - G)
    chords = [
        [261.63, 329.63, 392.00],  # C major (C4, E4, G4)
        [220.00, 261.63, 329.63],  # Am (A3, C4, E4)
        [174.61, 220.00, 261.63],  # F major (F3, A3, C4)
        [196.00, 246.94, 293.66],  # G major (G3, B3, D4)
    ]
    
    piano = np.zeros(int(sample_rate * duration))
    chord_duration = duration / len(chords)
    
    for i, chord_notes in enumerate(chords):
        chord = np.zeros(int(sample_rate * chord_duration))
        for freq in chord_notes:
            note = piano_note(freq, chord_duration, sample_rate)
            chord[:len(note)] += note * 0.3
        
        # Add to piano track
        start = int(i * chord_duration * sample_rate)
        end = start + len(chord)
        if end <= len(piano):
            piano[start:end] += chord
    
    return piano

def generate_electronic_tones(sample_rate=44100, duration=4):
    """Generate electronic synthesizer tones."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # LFO for modulation
    lfo_freq = 4  # 4 Hz LFO
    lfo = np.sin(2 * np.pi * lfo_freq * t)
    
    # Saw wave lead
    def saw_wave(frequency, t):
        """Generate a saw wave."""
        return 2 * ((frequency * t) % 1) - 1
    
    # Arpeggiator pattern
    arp_notes = [261.63, 329.63, 392.00, 523.25]  # C4, E4, G4, C5
    arp_duration = 0.125  # 16th notes at 120 BPM
    
    synth = np.zeros_like(t)
    
    # Generate arpeggiated pattern
    for i in range(int(duration / arp_duration)):
        note_freq = arp_notes[i % len(arp_notes)]
        start_idx = int(i * arp_duration * sample_rate)
        end_idx = min(start_idx + int(arp_duration * sample_rate), len(t))
        
        if start_idx < len(t):
            t_slice = t[start_idx:end_idx] - t[start_idx]
            
            # Saw wave with PWM-like effect
            saw = saw_wave(note_freq, t_slice)
            
            # Add some harmonics
            saw += 0.3 * saw_wave(note_freq * 2, t_slice)
            saw += 0.15 * saw_wave(note_freq * 3, t_slice)
            
            # Apply envelope
            env = create_envelope(arp_duration, sample_rate,
                                attack=0.001, decay=0.05,
                                sustain=0.3, release=0.1)
            
            if len(env) > len(saw):
                env = env[:len(saw)]
            else:
                saw = saw[:len(env)]
            
            synth[start_idx:start_idx + len(saw)] += saw * env * 0.4
    
    # Add filter sweep effect using LFO
    # Simple resonant filter simulation
    cutoff_base = 1000
    cutoff_mod = 800 * (lfo + 1) / 2
    cutoff = cutoff_base + cutoff_mod
    
    # Apply time-varying filter
    filtered_synth = np.zeros_like(synth)
    for i in range(len(synth)):
        if i > 0:
            # Simple one-pole low-pass filter
            alpha = 2 * np.pi * cutoff[i] / sample_rate
            alpha = alpha / (alpha + 1)
            filtered_synth[i] = alpha * synth[i] + (1 - alpha) * filtered_synth[i-1]
    
    # Add some delay/echo
    delay_samples = int(0.375 * sample_rate)  # Dotted quarter note delay
    delayed = np.zeros_like(filtered_synth)
    delayed[delay_samples:] = filtered_synth[:-delay_samples] * 0.4
    
    electronic = filtered_synth + delayed
    
    return electronic

def create_full_mix(sample_rate=44100, duration=4):
    """Create a full mix with all instruments."""
    print("Generating drum beat...")
    drums = generate_drum_beat(sample_rate, bpm=120, bars=int(duration/2))
    
    print("Generating guitar riff...")
    guitar = generate_guitar_riff(sample_rate, duration)
    
    print("Generating piano...")
    piano = generate_piano(sample_rate, duration)
    
    print("Generating electronic tones...")
    electronic = generate_electronic_tones(sample_rate, duration)
    
    # Ensure all tracks are the same length
    max_length = max(len(drums), len(guitar), len(piano), len(electronic))
    
    # Pad tracks if necessary
    if len(drums) < max_length:
        drums = np.pad(drums, (0, max_length - len(drums)))
    if len(guitar) < max_length:
        guitar = np.pad(guitar, (0, max_length - len(guitar)))
    if len(piano) < max_length:
        piano = np.pad(piano, (0, max_length - len(piano)))
    if len(electronic) < max_length:
        electronic = np.pad(electronic, (0, max_length - len(electronic)))
    
    # Mix all tracks with different levels
    mix = (drums * 0.8 + 
           guitar * 0.6 + 
           piano * 0.5 + 
           electronic * 0.4)
    
    # Apply compression
    threshold = 0.7
    ratio = 4
    compressed = np.where(
        np.abs(mix) > threshold,
        np.sign(mix) * (threshold + (np.abs(mix) - threshold) / ratio),
        mix
    )
    
    # Final limiting
    final_mix = np.tanh(compressed * 0.8) * 0.9
    
    # Add subtle reverb
    # reverb_time = 0.5
    # reverb_samples = int(sample_rate * reverb_time)
    # impulse = np.random.randn(reverb_samples) * np.exp(-3 * np.linspace(0, 1, reverb_samples))
    # impulse = impulse * 0.05
    
    # final_mix = signal.convolve(final_mix, impulse, mode='same')
    
    # Final normalization
    final_mix = final_mix / np.max(np.abs(final_mix)) * 0.95
    
    return final_mix

def main():

    sample_rate = 44100
    
    # Get output path from command line or use default
    output_path = '/tmp/audio_mix.wav' if len(sys.argv) < 2 else sys.argv[1]
    
    # Ensure output directory exists
    os.makedirs(os.path.dirname(output_path) or '.', exist_ok=True)
    
    # Generate the full mix
    print("\nGenerating full audio mix...")
    mix = create_full_mix(sample_rate, duration=4)
    
    # Save the mix
    sf.write(output_path, mix, sample_rate)
    print(f"\nSaved audio mix: {output_path}")



if __name__ == "__main__":
    main()
