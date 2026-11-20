#!/usr/bin/env python3
"""
Doom Retro Audio Synthesizer
Generates authentic 16-bit mono 44.1kHz PCM WAV sound effects using Python's standard library.
Strictly adheres to GL-0011: Zero single-letter variables anywhere.
"""

import math
import os
import struct
import wave

SAMPLE_RATE = 44100


def write_pcm_wav(file_path: str, samples: list[float]) -> None:
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with wave.open(file_path, "wb") as wave_file:
        wave_file.setnchannels(1)  # Mono
        wave_file.setsampwidth(2)  # 16-bit PCM
        wave_file.setframerate(SAMPLE_RATE)
        packed_frames = bytearray()
        for sample_float in samples:
            clamped_float = max(-1.0, min(1.0, sample_float))
            sample_integer = int(clamped_float * 32767.0)
            packed_frames.extend(struct.pack("<h", sample_integer))
        wave_file.writeframes(packed_frames)


def pseudo_noise(noise_seed: int) -> tuple[float, int]:
    next_seed = (noise_seed * 1103515245 + 12345) & 0x7FFFFFFF
    normalized_float = (float(next_seed) / 1073741824.0) - 1.0
    return normalized_float, next_seed


def generate_pistol_shot() -> list[float]:
    duration_seconds = 0.22
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 42817
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        amplitude_envelope = math.exp(-progress_ratio * 16.0)
        frequency = 750.0 * math.exp(-progress_ratio * 12.0) + 120.0
        sine_phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(sine_phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.55 * tone_sample + 0.45 * noise_sample) * amplitude_envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_shotgun_blast() -> list[float]:
    duration_seconds = 0.65
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 88412
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        if progress_ratio < 0.45:
            blast_progress = progress_ratio / 0.45
            blast_envelope = math.exp(-blast_progress * 7.0)
            bass_frequency = 180.0 * math.exp(-blast_progress * 9.0) + 40.0
            bass_phase = 2.0 * math.pi * bass_frequency * (float(sample_index) / float(SAMPLE_RATE))
            bass_sample = math.sin(bass_phase)
            noise_sample, noise_seed = pseudo_noise(noise_seed)
            composite_sample = (0.40 * bass_sample + 0.60 * noise_sample) * blast_envelope
        else:
            pump_progress = (progress_ratio - 0.45) / 0.55
            pump_envelope = math.sin(pump_progress * math.pi) * 0.40
            click_freq = 600.0 if pump_progress < 0.5 else 900.0
            click_phase = 2.0 * math.pi * click_freq * (float(sample_index) / float(SAMPLE_RATE))
            composite_sample = math.sin(click_phase) * pump_envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_chaingun_burst() -> list[float]:
    duration_seconds = 0.14
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 19482
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        amplitude_envelope = math.exp(-progress_ratio * 18.0)
        frequency = 600.0 * math.exp(-progress_ratio * 15.0) + 140.0
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.50 * tone_sample + 0.50 * noise_sample) * amplitude_envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_rocket_launch() -> list[float]:
    duration_seconds = 0.45
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 77123
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.sin(progress_ratio * math.pi * 0.5) * math.exp(-progress_ratio * 2.5)
        frequency = 220.0 + (progress_ratio * 380.0)
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.35 * tone_sample + 0.65 * noise_sample) * envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_rocket_explosion() -> list[float]:
    duration_seconds = 0.85
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 99231
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.exp(-progress_ratio * 4.5)
        sub_bass_frequency = 90.0 * math.exp(-progress_ratio * 6.0) + 30.0
        sub_phase = 2.0 * math.pi * sub_bass_frequency * (float(sample_index) / float(SAMPLE_RATE))
        sub_sample = math.sin(sub_phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.50 * sub_sample + 0.50 * noise_sample) * envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_imp_fireball() -> list[float]:
    duration_seconds = 0.38
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 33814
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.exp(-progress_ratio * 5.0)
        frequency = 420.0 * math.sin(progress_ratio * 12.0) + 500.0
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.30 * tone_sample + 0.70 * noise_sample) * envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_zombie_groan() -> list[float]:
    duration_seconds = 0.55
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.sin(progress_ratio * math.pi) * 0.75
        frequency = 160.0 - (progress_ratio * 40.0) + (math.sin(progress_ratio * 30.0) * 15.0)
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        sawtooth_sample = (2.0 * (phase / (2.0 * math.pi) % 1.0)) - 1.0
        composite_sample = sawtooth_sample * envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_demon_bite() -> list[float]:
    duration_seconds = 0.28
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 55198
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.exp(-progress_ratio * 12.0)
        frequency = 280.0 * math.exp(-progress_ratio * 8.0) + 80.0
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.60 * tone_sample + 0.40 * noise_sample) * envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_player_pain() -> list[float]:
    duration_seconds = 0.25
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.exp(-progress_ratio * 9.0)
        frequency = 210.0 - (progress_ratio * 90.0)
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(phase)
        composite_sample = tone_sample * envelope * 0.85
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_door_open() -> list[float]:
    duration_seconds = 0.60
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    noise_seed = 66291
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.sin(progress_ratio * math.pi) * 0.65
        frequency = 90.0 + (progress_ratio * 25.0)
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        tone_sample = math.sin(phase)
        noise_sample, noise_seed = pseudo_noise(noise_seed)
        composite_sample = (0.50 * tone_sample + 0.50 * noise_sample) * envelope
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_switch_click() -> list[float]:
    duration_seconds = 0.12
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.exp(-progress_ratio * 25.0)
        frequency = 1200.0 if progress_ratio < 0.5 else 800.0
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        composite_sample = math.sin(phase) * envelope * 0.80
        sample_buffer.append(composite_sample)
    return sample_buffer


def generate_item_pickup() -> list[float]:
    duration_seconds = 0.22
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_buffer = []
    for sample_index in range(total_samples):
        progress_ratio = float(sample_index) / float(total_samples)
        envelope = math.exp(-progress_ratio * 7.0)
        frequency = 440.0 if progress_ratio < 0.4 else (660.0 if progress_ratio < 0.7 else 880.0)
        phase = 2.0 * math.pi * frequency * (float(sample_index) / float(SAMPLE_RATE))
        composite_sample = math.sin(phase) * envelope * 0.60
        sample_buffer.append(composite_sample)
    return sample_buffer


def main() -> None:
    target_directory = "examples/doom/resources/audio"
    audio_generators = [
        ("pistol.wav", generate_pistol_shot),
        ("shotgun.wav", generate_shotgun_blast),
        ("chaingun.wav", generate_chaingun_burst),
        ("rocket_launch.wav", generate_rocket_launch),
        ("rocket_explosion.wav", generate_rocket_explosion),
        ("imp_fireball.wav", generate_imp_fireball),
        ("zombie_groan.wav", generate_zombie_groan),
        ("demon_bite.wav", generate_demon_bite),
        ("player_pain.wav", generate_player_pain),
        ("door_open.wav", generate_door_open),
        ("switch_click.wav", generate_switch_click),
        ("item_pickup.wav", generate_item_pickup),
    ]

    for file_name, generator_function in audio_generators:
        destination_path = os.path.join(target_directory, file_name)
        waveform_samples = generator_function()
        write_pcm_wav(destination_path, waveform_samples)
        print(f"Synthesized: {destination_path} ({len(waveform_samples)} samples)")


if __name__ == "__main__":
    main()
