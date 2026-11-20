#!/usr/bin/env python3
"""Generates retro arcade WAV audio assets for the Solix Invaders showcase game.

Zero external dependencies: uses only Python standard library `wave`, `struct`, and `math`.
"""

import math
import os
import random
import struct
import wave

SAMPLE_RATE = 44100


def write_pcm_wav(file_path: str, samples: list[float]) -> None:
    """Writes a 16-bit mono PCM WAV file normalized to [-1.0, 1.0]."""
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with wave.open(file_path, "wb") as wav_writer:
        wav_writer.setnchannels(1)
        wav_writer.setsampwidth(2)
        wav_writer.setframerate(SAMPLE_RATE)
        packed_frames = bytearray()
        for sample_value in samples:
            clamped_sample = max(-1.0, min(1.0, sample_value))
            scaled_int = int(clamped_sample * 32767.0)
            packed_frames.extend(struct.pack("<h", scaled_int))
        wav_writer.writeframes(packed_frames)


def generate_laser(file_path: str) -> None:
    """Synthesizes a retro laser beam firing sound (fast pitch sweep down)."""
    duration_seconds = 0.14
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    current_phase = 0.0

    for sample_index in range(total_samples):
        time_fraction = sample_index / float(total_samples)
        current_frequency = 980.0 * (1.0 - time_fraction * 0.72)
        current_phase += 2.0 * math.pi * current_frequency / SAMPLE_RATE

        # Square wave with slight smoothing
        raw_wave = 1.0 if math.sin(current_phase) > 0.0 else -1.0
        envelope = 1.0 - time_fraction
        sample_list.append(raw_wave * envelope * 0.7)

    write_pcm_wav(file_path, sample_list)


def generate_alien_laser(file_path: str) -> None:
    """Synthesizes an alien plasma bomb drop sound (buzzing frequency drop)."""
    duration_seconds = 0.20
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    current_phase = 0.0

    for sample_index in range(total_samples):
        time_fraction = sample_index / float(total_samples)
        vibrato = math.sin(2.0 * math.pi * 32.0 * (sample_index / SAMPLE_RATE)) * 45.0
        current_frequency = (620.0 * (1.0 - time_fraction * 0.65)) + vibrato
        current_phase += 2.0 * math.pi * current_frequency / SAMPLE_RATE

        # Sawtooth wave
        saw_wave = (2.0 * (current_phase / (2.0 * math.pi) % 1.0)) - 1.0
        envelope = math.exp(-2.5 * time_fraction)
        sample_list.append(saw_wave * envelope * 0.6)

    write_pcm_wav(file_path, sample_list)


def generate_explosion(file_path: str) -> None:
    """Synthesizes a classic arcade noise explosion burst."""
    duration_seconds = 0.32
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    random_state = random.Random(42)

    low_pass_filter = 0.0
    for sample_index in range(total_samples):
        time_fraction = sample_index / float(total_samples)
        noise_value = random_state.uniform(-1.0, 1.0)

        # Simple single-pole low pass filter to give explosion body
        filter_alpha = 0.15 + (1.0 - time_fraction) * 0.35
        low_pass_filter = (filter_alpha * noise_value) + ((1.0 - filter_alpha) * low_pass_filter)

        envelope = math.exp(-4.2 * time_fraction)
        sample_list.append(low_pass_filter * envelope * 0.9)

    write_pcm_wav(file_path, sample_list)


def generate_invader_step(file_path: str, base_frequency: float) -> None:
    """Synthesizes one note of the classic 4-note marching cadence."""
    duration_seconds = 0.075
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    current_phase = 0.0

    for sample_index in range(total_samples):
        time_fraction = sample_index / float(total_samples)
        current_phase += 2.0 * math.pi * base_frequency / SAMPLE_RATE

        square_wave = 1.0 if math.sin(current_phase) > 0.0 else -1.0
        envelope = math.exp(-5.0 * time_fraction)
        sample_list.append(square_wave * envelope * 0.75)

    write_pcm_wav(file_path, sample_list)


def generate_ufo(file_path: str) -> None:
    """Synthesizes the undulating warble of the Mystery Mothership."""
    duration_seconds = 0.45
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    current_phase = 0.0

    for sample_index in range(total_samples):
        current_time = sample_index / float(SAMPLE_RATE)
        lfo_frequency = 4.5
        frequency_modulation = math.sin(2.0 * math.pi * lfo_frequency * current_time) * 90.0
        carrier_frequency = 480.0 + frequency_modulation
        current_phase += 2.0 * math.pi * carrier_frequency / SAMPLE_RATE

        triangle_wave = 2.0 * abs((2.0 * (current_phase / (2.0 * math.pi) % 1.0)) - 1.0) - 1.0
        sample_list.append(triangle_wave * 0.65)

    write_pcm_wav(file_path, sample_list)


def generate_player_death(file_path: str) -> None:
    """Synthesizes the deep disintegrating explosion when the player loses a life."""
    duration_seconds = 0.55
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    random_state = random.Random(1337)
    current_phase = 0.0

    for sample_index in range(total_samples):
        time_fraction = sample_index / float(total_samples)
        current_frequency = 320.0 * (1.0 - time_fraction * 0.8)
        current_phase += 2.0 * math.pi * current_frequency / SAMPLE_RATE

        sine_wave = math.sin(current_phase)
        noise_value = random_state.uniform(-1.0, 1.0) * (1.0 - time_fraction * 0.5)
        combined_signal = (sine_wave * 0.45) + (noise_value * 0.55)

        envelope = math.exp(-3.5 * time_fraction)
        sample_list.append(combined_signal * envelope * 0.95)

    write_pcm_wav(file_path, sample_list)


def generate_shield_hit(file_path: str) -> None:
    """Synthesizes the short metallic snap when a defensive bunker block is hit."""
    duration_seconds = 0.05
    total_samples = int(SAMPLE_RATE * duration_seconds)
    sample_list = []
    random_state = random.Random(777)

    for sample_index in range(total_samples):
        time_fraction = sample_index / float(total_samples)
        noise_value = random_state.uniform(-1.0, 1.0)
        envelope = math.exp(-12.0 * time_fraction)
        sample_list.append(noise_value * envelope * 0.8)

    write_pcm_wav(file_path, sample_list)


def main() -> None:
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    audio_directory = os.path.join(project_root, "resources", "audio")

    generate_laser(os.path.join(audio_directory, "laser.wav"))
    generate_alien_laser(os.path.join(audio_directory, "alien_laser.wav"))
    generate_explosion(os.path.join(audio_directory, "explosion.wav"))
    generate_invader_step(os.path.join(audio_directory, "invader_step1.wav"), 175.0)
    generate_invader_step(os.path.join(audio_directory, "invader_step2.wav"), 160.0)
    generate_invader_step(os.path.join(audio_directory, "invader_step3.wav"), 145.0)
    generate_invader_step(os.path.join(audio_directory, "invader_step4.wav"), 130.0)
    generate_ufo(os.path.join(audio_directory, "ufo.wav"))
    generate_player_death(os.path.join(audio_directory, "player_death.wav"))
    generate_shield_hit(os.path.join(audio_directory, "shield_hit.wav"))
    print(f"Generated 10 retro arcade audio files in '{audio_directory}'")


if __name__ == "__main__":
    main()
