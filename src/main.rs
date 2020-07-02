//#![allow(unused_imports, dead_code)]
extern crate anyhow;
extern crate crossbeam_channel;
extern crate libloading;
extern crate num_traits as num;
use std::time::Instant;
pub mod types;
pub mod waves;
use types::*;
pub mod consts;
mod live_output;
use consts::*;

fn freq(p: &Note<f64, f64>, waveform: fn(f64) -> f64) -> Wave<f64> {
    match p {
        Note {
            silent: true,
            duration,
            ..
        } => return vec![0f64; (duration * SAMPLERATE) as usize],
        Note {
            silent: false,
            freq,
            duration,
        } => {
            let volume = 0.1;
            let lfo = LFO::new(0., 0.);
            return (0..=(duration * SAMPLERATE) as usize)
                .map(|x| x as f64 / SAMPLERATE)
                .map(|t| {
                    let attack = 0.01;
                    let decay = 0.1;
                    let mut alpha = if t <= attack {
                        //attack
                        t / attack
                    } else if t >= (duration - decay) {
                        //release
                        (duration - t) / decay
                    } else {
                        //sustain
                        1.
                    };
                    alpha *= 1.0;
                    (waveform(lfo.apply(*freq, t)) * alpha) * volume
                })
                .collect();
        }
    }
}
struct Argument {
    full: String,
    short: String,
    help: String,
    arg_type: ArgState,
}

enum ArgState {
    /// "-arg" vs ""
    Presence(bool),
    /// -arg value
    Following(Option<String>),
}

struct ArgStruct {
    args: Vec<Argument>,
}
use libloading::{Library, Symbol};
use std::collections::HashMap;
/// Loads instruments from the given library
fn load_instruments(
    lib: &'static mut Library,
    symbols: &[String],
) -> HashMap<String, Box<Symbol<'static, extern "C" fn(f64) -> f64>>> {
    let mut ret: HashMap<String, Box<Symbol<'static, extern "C" fn(f64) -> f64>>> = HashMap::new();
    for symbol in symbols {
        unsafe {
            match lib.get::<extern "C" fn(f64) -> f64>(symbol.as_bytes()) {
                Ok(func) => {
                    ret.insert(String::from(symbol), Box::new(func));
                }
                Err(_e) => { /* TODO: Log error */ }
            }
        }
    }
    ret
}

/// Load all libraries under specified path
fn load_libraries(path: &str) -> anyhow::Result<Vec<(&'static mut Library, &'static [String])>> {
    let mut ret: Vec<(&'static mut Library, &'static [String])> = Vec::new();
    let dir = std::fs::read_dir(path).unwrap();
    let libraries = dir.filter_map(|e| {
        // FIXME: I need to handle errors better, for now let's hope that the user won't delete the program while it's starting
        let e = e.unwrap();
        let ft = e.file_type().unwrap();
        match ft.is_file() {
            true => Some(e.path()),
            false => None,
        }
    });
    for library in libraries {
        match Library::new(&library) {
            Ok(mut lib) => {
                let maybe_init: Result<
                    Symbol<extern "C" fn() -> &'static [String]>,
                    libloading::Error,
                >;
                unsafe {
                    maybe_init = lib.get::<extern "C" fn() -> &'static [String]>(b"init");
                }
                match maybe_init {
                    Ok(init) => {
                        let instruments = init();
                        ret.push((&mut lib, instruments));
                    }
                    Err(e) => eprintln!("Failed to initialize library {:?} \n {}", library, e),
                };
            }
            Err(e) => eprintln!("Failed to load library {:?} \n {}", library, e),
        }
    }
    match ret.len() {
        0 => Err(anyhow::anyhow!("No libraries could be loaded")),
        _ => Ok(ret),
    }
}

fn main() -> anyhow::Result<()> {
    let instrument_libraries = load_libraries("./instruments")?;
    let instruments = load_instruments(instrument_libraries[0].0, instrument_libraries[0].1);
    // load instrument libraries
    /* PSEUDOCODE
    for instrument.dll in ./instruments
        let lib = lib::Library::new("./instruments/instrument.dll")?;
        let func: lib::Symbol<unsafe extern fn() -> u32> = lib.get(b"init\0")?;

    */
    use std::thread as th;
    // parse arguments
    // start UI thread (using druid)
    // start synth and output threads
    // or parse input file, feed notes to synth and write audio to ouptut file
    // pipes?
    let outp = std::env::args()
        .find(|a| a.ends_with(".bin"))
        .expect("No valid output file specified");
    let t1 = Instant::now();
    /* let wave = &[
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(5., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::silent(1.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(2.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(5., 0.25),
        Note::silent(1.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
    ]; */
    let wave = &[Note::new(440., 1.), Note::new(450., 1.)];
    wave.iter()
        .map(|note| {
            freq(note, |hertz| {
                use std::f64;
                f64::sin(hertz)
            })
        })
        .flatten()
        .collect::<Wave<f64>>()
        .write(&outp);
    println!("{:?}", t1.elapsed());
    Ok(())
}

/* fn mkfn(i: usize) -> impl Fn((f64, f64)) -> (f64, f64) {
    move |input| (input.0 * i as f64, input.1)
}
fn foo() -> f64 {
    let hz: f64 = 440.;
    let t: f64 = 0.1;
    let chains = vec![
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
    ];
    chains
        .iter()
        .map(|c| c.iter().fold((hz, t), |input: (f64, f64), fun| fun(input)))
        .map(|c: (f64, f64)| c.0)
        .sum()
} */
