use crate::library::{Album, Artist, Catalog, Track};

pub fn build_mock_catalog() -> Catalog {
    Catalog {
        artists: vec![
            Artist {
                name: "Étoiles Polaires".to_string(),
                albums: vec![
                    Album {
                        title: "Lueurs Nordiques".to_string(),
                        year: 2021,
                        tracks: vec![
                            Track {
                                number: 1,
                                title: "Halo".to_string(),
                                duration_secs: 213,
                            },
                            Track {
                                number: 2,
                                title: "Aurore".to_string(),
                                duration_secs: 189,
                            },
                            Track {
                                number: 3,
                                title: "Constellations".to_string(),
                                duration_secs: 241,
                            },
                        ],
                    },
                    Album {
                        title: "Marées Lunaires".to_string(),
                        year: 2024,
                        tracks: vec![
                            Track {
                                number: 1,
                                title: "Gravité Douce".to_string(),
                                duration_secs: 206,
                            },
                            Track {
                                number: 2,
                                title: "Résonance".to_string(),
                                duration_secs: 198,
                            },
                        ],
                    },
                ],
            },
            Artist {
                name: "Velours Analogiques".to_string(),
                albums: vec![Album {
                    title: "Polaroids".to_string(),
                    year: 2019,
                    tracks: vec![
                        Track {
                            number: 1,
                            title: "Instantané".to_string(),
                            duration_secs: 175,
                        },
                        Track {
                            number: 2,
                            title: "Développer".to_string(),
                            duration_secs: 232,
                        },
                    ],
                }],
            },
        ],
    }
}
