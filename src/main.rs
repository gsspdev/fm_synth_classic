use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::{self, Write};

#[cfg(not(target_arch = "wasm32"))]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// FM Synthesizer parameters
#[derive(Clone, Debug)]
struct FMParams {
    carrier_freq: f32,      // Carrier frequency in Hz
    modulator_freq: f32,    // Modulator frequency in Hz
    modulation_index: f32,  // Modulation depth
    amplitude: f32,         // Output amplitude (0.0 - 1.0)
}

impl Default for FMParams {
    fn default() -> Self {
        Self {
            carrier_freq: 440.0,
            modulator_freq: 220.0,
            modulation_index: 2.0,
            amplitude: 0.3,
        }
    }
}

/// Note structure for melodies
#[derive(Clone, Debug)]
struct Note {
    frequency: f32,
    duration_ms: u64,
}

/// FM Synthesizer oscillator
struct FMOscillator {
    sample_rate: f32,
    carrier_phase: f32,
    modulator_phase: f32,
    params: FMParams,
}

impl FMOscillator {
    fn new(sample_rate: f32, params: FMParams) -> Self {
        Self {
            sample_rate,
            carrier_phase: 0.0,
            modulator_phase: 0.0,
            params,
        }
    }

    fn next_sample(&mut self) -> f32 {
        let modulator = (2.0 * PI * self.modulator_phase).sin();
        let modulated_freq = self.params.carrier_freq * 
            (1.0 + self.params.modulation_index * modulator);
        let carrier = (2.0 * PI * self.carrier_phase).sin();
        
        self.carrier_phase += modulated_freq / self.sample_rate;
        self.modulator_phase += self.params.modulator_freq / self.sample_rate;
        
        if self.carrier_phase >= 1.0 {
            self.carrier_phase -= 1.0;
        }
        if self.modulator_phase >= 1.0 {
            self.modulator_phase -= 1.0;
        }
        
        carrier * self.params.amplitude
    }

    fn set_params(&mut self, params: FMParams) {
        self.params = params;
    }
}

/// ADSR Envelope generator
struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    
    sample_rate: f32,
    state: EnvelopeState,
    level: f32,
    time: f32,
}

#[derive(PartialEq)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Envelope {
    fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.5,
            sample_rate,
            state: EnvelopeState::Idle,
            level: 0.0,
            time: 0.0,
        }
    }

    fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.time = 0.0;
    }

    fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.time = 0.0;
        }
    }

    fn process(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        
        match self.state {
            EnvelopeState::Idle => {
                self.level = 0.0;
            }
            EnvelopeState::Attack => {
                self.level = self.time / self.attack;
                if self.time >= self.attack {
                    self.state = EnvelopeState::Decay;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Decay => {
                self.level = 1.0 - ((1.0 - self.sustain) * (self.time / self.decay));
                if self.time >= self.decay {
                    self.state = EnvelopeState::Sustain;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Sustain => {
                self.level = self.sustain;
            }
            EnvelopeState::Release => {
                self.level = self.sustain * (1.0 - (self.time / self.release));
                if self.time >= self.release {
                    self.state = EnvelopeState::Idle;
                    self.level = 0.0;
                }
            }
        }
        
        self.time += dt;
        self.level
    }
}

/// FM Synthesizer with envelope
struct FMSynth {
    oscillator: FMOscillator,
    envelope: Envelope,
}

impl FMSynth {
    fn new(sample_rate: f32, params: FMParams) -> Self {
        Self {
            oscillator: FMOscillator::new(sample_rate, params),
            envelope: Envelope::new(sample_rate),
        }
    }

    fn next_sample(&mut self) -> f32 {
        let osc_out = self.oscillator.next_sample();
        let env_out = self.envelope.process();
        osc_out * env_out
    }

    fn note_on(&mut self) {
        self.envelope.trigger();
    }

    fn note_off(&mut self) {
        self.envelope.release();
    }

    fn set_params(&mut self, params: FMParams) {
        self.oscillator.set_params(params);
    }
}

/// Preset definitions
fn get_presets() -> Vec<(&'static str, FMParams)> {
    vec![
        ("Bell", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 440.0,
            modulation_index: 7.0,
            amplitude: 0.3,
        }),
        ("Bass", FMParams {
            carrier_freq: 110.0,
            modulator_freq: 110.0,
            modulation_index: 1.5,
            amplitude: 0.5,
        }),
        ("Electric Piano", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 880.0,
            modulation_index: 3.0,
            amplitude: 0.4,
        }),
        ("Brass", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 440.0,
            modulation_index: 2.5,
            amplitude: 0.4,
        }),
        ("Organ", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 880.0,
            modulation_index: 1.0,
            amplitude: 0.4,
        }),
        ("Synth Lead", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 1320.0,
            modulation_index: 4.0,
            amplitude: 0.35,
        }),
        ("Marimba", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 440.0,
            modulation_index: 3.5,
            amplitude: 0.4,
        }),
        ("Strings", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 220.0,
            modulation_index: 0.8,
            amplitude: 0.3,
        }),
        ("Flute", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 440.0,
            modulation_index: 0.5,
            amplitude: 0.25,
        }),
        ("Metallic", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 567.0,
            modulation_index: 9.0,
            amplitude: 0.3,
        }),
        ("Glockenspiel", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 1760.0,
            modulation_index: 2.5,
            amplitude: 0.3,
        }),
        ("Wood Block", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 300.0,
            modulation_index: 12.0,
            amplitude: 0.4,
        }),
    ]
}

/// Note frequencies
fn note_freq(note: &str) -> f32 {
    match note {
        "C3" => 130.81, "C#3" => 138.59, "D3" => 146.83, "D#3" => 155.56, "E3" => 164.81,
        "F3" => 174.61, "F#3" => 185.00, "G3" => 196.00, "G#3" => 207.65, "A3" => 220.00,
        "A#3" => 233.08, "B3" => 246.94,
        "C4" => 261.63, "C#4" => 277.18, "D4" => 293.66, "D#4" => 311.13, "E4" => 329.63,
        "F4" => 349.23, "F#4" => 369.99, "G4" => 392.00, "G#4" => 415.30, "A4" => 440.00,
        "A#4" => 466.16, "B4" => 493.88,
        "C5" => 523.25, "C#5" => 554.37, "D5" => 587.33, "D#5" => 622.25, "E5" => 659.25,
        "F5" => 698.46, "F#5" => 739.99, "G5" => 783.99, "G#5" => 830.61, "A5" => 880.00,
        _ => 0.0, // Rest
    }
}

/// Melody definitions
fn get_melodies() -> Vec<(&'static str, Vec<(&'static str, u64)>)> {
    vec![
        ("Twinkle Twinkle", vec![
            ("C4", 500), ("C4", 500), ("G4", 500), ("G4", 500),
            ("A4", 500), ("A4", 500), ("G4", 1000),
            ("F4", 500), ("F4", 500), ("E4", 500), ("E4", 500),
            ("D4", 500), ("D4", 500), ("C4", 1000),
        ]),
        ("Happy Birthday", vec![
            ("C4", 250), ("C4", 250), ("D4", 500), ("C4", 500),
            ("F4", 500), ("E4", 1000),
            ("C4", 250), ("C4", 250), ("D4", 500), ("C4", 500),
            ("G4", 500), ("F4", 1000),
        ]),
        ("Ode to Joy", vec![
            ("E4", 500), ("E4", 500), ("F4", 500), ("G4", 500),
            ("G4", 500), ("F4", 500), ("E4", 500), ("D4", 500),
            ("C4", 500), ("C4", 500), ("D4", 500), ("E4", 500),
            ("E4", 750), ("D4", 250), ("D4", 1000),
        ]),
        ("Mary Had a Little Lamb", vec![
            ("E4", 500), ("D4", 500), ("C4", 500), ("D4", 500),
            ("E4", 500), ("E4", 500), ("E4", 1000),
            ("D4", 500), ("D4", 500), ("D4", 1000),
            ("E4", 500), ("G4", 500), ("G4", 1000),
        ]),
        ("Chromatic Scale", vec![
            ("C4", 200), ("C#4", 200), ("D4", 200), ("D#4", 200),
            ("E4", 200), ("F4", 200), ("F#4", 200), ("G4", 200),
            ("G#4", 200), ("A4", 200), ("A#4", 200), ("B4", 200),
            ("C5", 400),
        ]),
        ("Major Arpeggio", vec![
            ("C4", 300), ("E4", 300), ("G4", 300), ("C5", 300),
            ("G4", 300), ("E4", 300), ("C4", 600),
        ]),
        ("Minor Pentatonic", vec![
            ("A3", 400), ("C4", 400), ("D4", 400), ("E4", 400),
            ("G4", 400), ("A4", 400), ("G4", 400), ("E4", 400),
            ("D4", 400), ("C4", 400), ("A3", 800),
        ]),
        ("Jazz Lick", vec![
            ("C4", 200), ("E4", 200), ("G4", 200), ("A#4", 200),
            ("A4", 400), ("F4", 200), ("D4", 400),
            ("G4", 200), ("E4", 200), ("C4", 600),
        ]),
        ("Bach Invention", vec![
            ("C4", 200), ("D4", 200), ("E4", 200), ("F4", 200),
            ("D4", 200), ("E4", 200), ("C4", 400),
            ("G4", 200), ("F4", 200), ("E4", 200), ("D4", 200),
            ("B3", 200), ("C4", 600),
        ]),
        ("Synth Demo", vec![
            ("C4", 150), ("E4", 150), ("G4", 150), ("C5", 150),
            ("E5", 150), ("G5", 150), ("E5", 150), ("C5", 150),
            ("G4", 150), ("E4", 150), ("C4", 300),
            ("REST", 300),
            ("F4", 150), ("A4", 150), ("C5", 150), ("F5", 150),
            ("C5", 150), ("A4", 150), ("F4", 300),
        ]),
    ]
}

/// CLI interface
struct CLI {
    presets: Vec<(&'static str, FMParams)>,
    melodies: Vec<(&'static str, Vec<(&'static str, u64)>)>,
}

impl CLI {
    fn new() -> Self {
        Self {
            presets: get_presets(),
            melodies: get_melodies(),
        }
    }

    fn print_menu(&self) {
        println!("\n=== FM Synthesizer CLI ===");
        println!("Commands:");
        println!("  list presets  - Show all available presets");
        println!("  list melodies - Show all available melodies");
        println!("  play <preset> <melody> - Play a melody with a preset");
        println!("  demo - Play all presets with a scale");
        println!("  help - Show this menu");
        println!("  quit - Exit the program");
        println!();
    }

    fn list_presets(&self) {
        println!("\nAvailable Presets:");
        for (i, (name, _)) in self.presets.iter().enumerate() {
            println!("  {}. {}", i + 1, name);
        }
    }

    fn list_melodies(&self) {
        println!("\nAvailable Melodies:");
        for (i, (name, _)) in self.melodies.iter().enumerate() {
            println!("  {}. {}", i + 1, name);
        }
    }

    fn find_preset(&self, name: &str) -> Option<FMParams> {
        // Try by number first
        if let Ok(num) = name.parse::<usize>() {
            if num > 0 && num <= self.presets.len() {
                return Some(self.presets[num - 1].1.clone());
            }
        }
        
        // Try by name (case insensitive)
        self.presets.iter()
            .find(|(n, _)| n.to_lowercase() == name.to_lowercase())
            .map(|(_, p)| p.clone())
    }

    fn find_melody(&self, name: &str) -> Option<Vec<(&'static str, u64)>> {
        // Try by number first
        if let Ok(num) = name.parse::<usize>() {
            if num > 0 && num <= self.melodies.len() {
                return Some(self.melodies[num - 1].1.clone());
            }
        }
        
        // Try by name (case insensitive)
        self.melodies.iter()
            .find(|(n, _)| n.to_lowercase().contains(&name.to_lowercase()))
            .map(|(_, m)| m.clone())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_melody(preset: FMParams, melody: Vec<(&'static str, u64)>) -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .expect("No output device available");
    
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    
    let synth = Arc::new(Mutex::new(FMSynth::new(sample_rate, preset.clone())));
    let synth_clone = Arc::clone(&synth);
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut synth = synth_clone.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = synth.next_sample();
                }
            },
            |err| eprintln!("Error in audio stream: {}", err),
            None,
        )?,
        _ => panic!("Unsupported sample format"),
    };
    
    stream.play()?;
    
    for (note, duration) in melody {
        let freq = note_freq(note);
        if freq > 0.0 {
            let mut params = preset.clone();
            let freq_ratio = freq / 440.0;
            params.carrier_freq *= freq_ratio;
            params.modulator_freq *= freq_ratio;
            
            {
                let mut synth = synth.lock().unwrap();
                synth.set_params(params);
                synth.note_on();
            }
            
            std::thread::sleep(Duration::from_millis(duration * 80 / 100));
            
            {
                let mut synth = synth.lock().unwrap();
                synth.note_off();
            }
            
            std::thread::sleep(Duration::from_millis(duration * 20 / 100));
        } else {
            std::thread::sleep(Duration::from_millis(duration));
        }
    }
    
    std::thread::sleep(Duration::from_millis(500));
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let mut cli = CLI::new();
    cli.print_menu();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "list" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "presets" => cli.list_presets(),
                        "melodies" => cli.list_melodies(),
                        _ => println!("Unknown list command. Use 'list presets' or 'list melodies'"),
                    }
                } else {
                    println!("Usage: list <presets|melodies>");
                }
            }
            "play" => {
                if parts.len() >= 3 {
                    let preset_name = parts[1];
                    let melody_name = parts[2..].join(" ");
                    
                    match (cli.find_preset(preset_name), cli.find_melody(&melody_name)) {
                        (Some(preset), Some(melody)) => {
                            println!("Playing '{}' melody with '{}' preset...", melody_name, preset_name);
                            play_melody(preset, melody)?;
                            println!("Done!");
                        }
                        (None, _) => println!("Preset '{}' not found. Use 'list presets' to see available options.", preset_name),
                        (_, None) => println!("Melody '{}' not found. Use 'list melodies' to see available options.", melody_name),
                    }
                } else {
                    println!("Usage: play <preset> <melody>");
                    println!("Example: play bell twinkle");
                    println!("Example: play 1 3");
                }
            }
            "demo" => {
                println!("Playing demo with all presets...");
                let scale = vec![
                    ("C4", 300), ("D4", 300), ("E4", 300), ("F4", 300),
                    ("G4", 300), ("A4", 300), ("B4", 300), ("C5", 600),
                ];
                for (name, preset) in &cli.presets {
                    println!("  Playing: {}", name);
                    play_melody(preset.clone(), scale.clone())?;
                }
                println!("Demo complete!");
            }
            "help" => cli.print_menu(),
            "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }
    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("This binary is not meant to be run in WebAssembly. Use the web interface instead.");
}
