/*
**  _                                              _      ___    ___
** | |                                            | |    |__ \  / _ \
** | |_Created _       _ __   _ __    ___    __ _ | |__     ) || (_) |
** | '_ \ | | | |     | '_ \ | '_ \  / _ \  / _` || '_ \   / /  \__, |
** | |_) || |_| |     | | | || | | || (_) || (_| || | | | / /_    / /
** |_.__/  \__, |     |_| |_||_| |_| \___/  \__,_||_| |_||____|  /_/
**          __/ |     on 2026-11-22.
**         |___/
**
**
** Test aléatoire de l'insertion : séquences réalistes de textes provisoires, révisés, vides et définitifs.
*/

use slate::voice::inserter::Inserter;
use slate::voice::text_target::StringTarget;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % bound
    }
}

const WORDS: [&str; 8] = [
    "Alors.", "Non,", "c'est", "pas", "ce", "qu'on", "dit", "été",
];

fn sentence(rng: &mut Lcg, words: usize) -> String {
    (0..words)
        .map(|_| WORDS[rng.next(8) as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_sequence(seed: u64) -> String {
    let mut rng = Lcg(seed);
    let mut target = StringTarget::new("# Titre\n\n", None);
    let mut inserter = Inserter::default();
    let mut current = String::new();
    for _ in 0..40 {
        match rng.next(10) {
            0..=5 => {
                let extra = sentence(&mut rng, 1);
                current = if current.is_empty() {
                    extra
                } else {
                    format!("{current} {extra}")
                };
                inserter.show_partial(&mut target, &current);
            }
            6 => {
                inserter.show_partial(&mut target, "");
            }
            7 => {
                let cut = current.rfind(' ').unwrap_or(0);
                current.truncate(cut);
                inserter.show_partial(&mut target, &current);
            }
            _ => {
                inserter.commit(&mut target, &current);
                current.clear();
            }
        }
    }
    target.text().to_string()
}

#[test]
fn test_random_dictations_never_create_stray_blanks() {
    for seed in 0..3000 {
        let text = run_sequence(seed);
        let body = text.strip_prefix("# Titre\n\n").unwrap_or(&text);
        assert!(
            !body.contains("  "),
            "seed {seed} : espaces doublées dans {body:?}"
        );
        assert!(
            !body.starts_with(' '),
            "seed {seed} : espace de tête dans {body:?}"
        );
    }
}
