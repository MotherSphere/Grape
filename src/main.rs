mod library;
mod mock_catalog;
mod player;
mod playlist;

use crate::library::Catalog;
use crate::player::{NowPlaying, PlayerState};

fn main() {
    let catalog = mock_catalog::build_mock_catalog();
    let player_state = build_player_state(&catalog);

    render_library(&catalog);
    println!();
    render_player(&player_state);
}

fn build_player_state(catalog: &Catalog) -> PlayerState {
    let now_playing = catalog.first_track().map(|(artist, album, track)| NowPlaying {
        artist: artist.name.clone(),
        album: album.title.clone(),
        title: track.title.clone(),
        duration_secs: track.duration_secs,
    });

    PlayerState {
        now_playing,
        is_playing: true,
        position_secs: 42,
    }
}

fn render_library(catalog: &Catalog) {
    println!("== Bibliothèque ==");
    for artist in &catalog.artists {
        println!("Artiste: {}", artist.name);
        for album in &artist.albums {
            println!("  Album: {} ({})", album.title, album.year);
            for track in &album.tracks {
                println!(
                    "    {:02}. {} ({}s)",
                    track.number, track.title, track.duration_secs
                );
            }
        }
    }
}

fn render_player(player_state: &PlayerState) {
    println!("== Lecture ==");
    if let Some(track) = &player_state.now_playing {
        println!("Titre  : {}", track.title);
        println!("Artiste: {}", track.artist);
        println!("Album  : {}", track.album);
        println!(
            "Progression: {}s / {}s",
            player_state.position_secs, track.duration_secs
        );
        println!("Statut: {}", if player_state.is_playing { "Lecture" } else { "Pause" });
    } else {
        println!("Aucune piste sélectionnée");
    }
}
