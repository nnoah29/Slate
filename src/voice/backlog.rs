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
** Réserve audio : garde le son dit pendant le chargement du modèle pour le rejouer dans l'ordre.
*/

use std::collections::VecDeque;

/// File de paquets audio bornée en nombre d'échantillons ; le plus ancien est écarté au-delà.
#[derive(Debug)]
pub struct Backlog {
    chunks: VecDeque<Vec<f32>>,
    samples: usize,
    capacity: usize,
}

impl Backlog {
    /// Réserve pouvant contenir `capacity` échantillons.
    pub fn new(capacity: usize) -> Self {
        Self {
            chunks: VecDeque::new(),
            samples: 0,
            capacity,
        }
    }

    /// Ajoute un paquet ; écarte les plus anciens tant que la capacité est dépassée (le dernier reste).
    pub fn push(&mut self, chunk: Vec<f32>) {
        self.samples += chunk.len();
        self.chunks.push_back(chunk);
        while self.samples > self.capacity && self.chunks.len() > 1 {
            if let Some(old) = self.chunks.pop_front() {
                self.samples -= old.len();
            }
        }
    }

    /// Rend tous les paquets dans l'ordre et vide la réserve.
    pub fn drain(&mut self) -> Vec<Vec<f32>> {
        self.samples = 0;
        self.chunks.drain(..).collect()
    }
}
