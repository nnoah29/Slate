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
** Machine à états d'Espace maintenu : appui court = espace, maintien = dictée tant que la touche est tenue.
*/

use std::time::Duration;

const DEFAULT_HOLD_MS: u64 = 400;

/// Ce que l'interface doit faire d'un événement clavier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PttAction {
    PassThrough,
    Swallow,
    InsertSpace,
    StartDictation,
    StopDictation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Pending,
    Dictating,
    Flushed,
}

/// Suit la touche Espace : indécise jusqu'au relâchement ou au délai de maintien.
#[derive(Debug)]
pub struct PushToTalk {
    state: State,
    hold_ms: u64,
}

impl Default for PushToTalk {
    fn default() -> Self {
        Self::new(DEFAULT_HOLD_MS)
    }
}

impl PushToTalk {
    /// Machine avec un délai de maintien en millisecondes.
    pub fn new(hold_ms: u64) -> Self {
        Self {
            state: State::Idle,
            hold_ms,
        }
    }

    /// Délai de maintien après lequel la dictée démarre.
    pub fn hold_delay(&self) -> Duration {
        Duration::from_millis(self.hold_ms)
    }

    /// Vrai tant que la machine garde la main sur la touche (indécise, en dictée, ou espace déjà écrite).
    pub fn is_active(&self) -> bool {
        self.state != State::Idle
    }

    /// Espace enfoncée ; `eligible` dit si la dictée est possible (sans modificateur, en édition, modèle présent...).
    pub fn key_down(&mut self, eligible: bool, repeat: bool) -> PttAction {
        match (self.state, eligible, repeat) {
            (State::Idle, true, false) => {
                self.state = State::Pending;
                PttAction::Swallow
            }
            (State::Idle, _, _) => PttAction::PassThrough,
            _ => PttAction::Swallow,
        }
    }

    /// Le délai de maintien est écoulé : la dictée démarre si la touche est encore indécise.
    pub fn hold_elapsed(&mut self) -> PttAction {
        if self.state != State::Pending {
            return PttAction::PassThrough;
        }
        self.state = State::Dictating;
        PttAction::StartDictation
    }

    /// Espace relâchée : espace écrite si appui court, fin de dictée si maintien.
    pub fn key_up(&mut self) -> PttAction {
        let action = match self.state {
            State::Idle => PttAction::PassThrough,
            State::Pending => PttAction::InsertSpace,
            State::Dictating => PttAction::StopDictation,
            State::Flushed => PttAction::Swallow,
        };
        self.state = State::Idle;
        action
    }

    /// Une autre touche est pressée : l'espace en attente est écrite d'abord, la frappe suit.
    pub fn other_key(&mut self) -> PttAction {
        if self.state != State::Pending {
            return PttAction::PassThrough;
        }
        self.state = State::Flushed;
        PttAction::InsertSpace
    }

    /// Abandonne tout suivi (perte de focus, changement de mode, dictée annulée).
    pub fn cancel(&mut self) {
        self.state = State::Idle;
    }
}
