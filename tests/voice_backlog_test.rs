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
** Tests de la réserve audio : le son dit pendant le chargement du modèle n'est pas perdu (T7).
*/

use slate::voice::backlog::Backlog;

#[test]
fn test_backlog_returns_chunks_in_order_and_empties() {
    let mut backlog = Backlog::new(100);
    backlog.push(vec![1.0; 10]);
    backlog.push(vec![2.0; 10]);
    let drained = backlog.drain();
    assert_eq!(drained, vec![vec![1.0; 10], vec![2.0; 10]]);
    assert!(backlog.drain().is_empty());
}

#[test]
fn test_backlog_drops_oldest_audio_beyond_capacity() {
    let mut backlog = Backlog::new(25);
    for value in [1.0, 2.0, 3.0] {
        backlog.push(vec![value; 10]);
    }
    assert_eq!(backlog.drain(), vec![vec![2.0; 10], vec![3.0; 10]]);
}

#[test]
fn test_backlog_keeps_a_single_oversized_chunk() {
    let mut backlog = Backlog::new(5);
    backlog.push(vec![9.0; 10]);
    assert_eq!(backlog.drain(), vec![vec![9.0; 10]]);
}
