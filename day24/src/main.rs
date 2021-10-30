use std::collections::{HashMap, HashSet};
use std::iter;
use std::ops::{Add};
use std::str::FromStr;

use anyhow::Result;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct HexVector {
    x: i32,
    y: i32,
    z: i32,
}

impl HexVector {
    fn neighbors(&self) -> impl Iterator<Item=HexVector> + '_ {
        const DIRS: [HexVector; 6] = [
            HexVector {
                x: 1,
                y: -1,
                z: 0,
            },
            HexVector {
                x: -1,
                y: 1,
                z: 0,
            },
            HexVector {
                x: 0,
                y: -1,
                z: 1,
            },
            HexVector {
                x: -1,
                y: 0,
                z: 1,
            },
            HexVector {
                x: 1,
                y: 0,
                z: -1,
            },
            HexVector {
                x: 0,
                y: 1,
                z: -1,
            }
        ];

        DIRS.iter()
            .map(move |d| d + self)
    }
}

impl Add for &HexVector {
    type Output = HexVector;

    fn add(self, rhs: Self) -> Self::Output {
        HexVector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl FromStr for HexVector {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = regex::Regex::new(r"e|w|se|sw|ne|nw").unwrap();
        let mut ret = HexVector {
            x: 0,
            y: 0,
            z: 0,
        };
        for cap in pattern.captures_iter(s) {
            let dir = match &cap[0] {
                "e" => HexVector {
                    x: 1,
                    y: -1,
                    z: 0,
                },
                "w" => HexVector {
                    x: -1,
                    y: 1,
                    z: 0,
                },
                "se" => HexVector {
                    x: 0,
                    y: -1,
                    z: 1,
                },
                "sw" => HexVector {
                    x: -1,
                    y: 0,
                    z: 1,
                },
                "ne" => HexVector {
                    x: 1,
                    y: 0,
                    z: -1,
                },
                "nw" => HexVector {
                    x: 0,
                    y: 1,
                    z: -1,
                },
                _ => panic!("bad direction")
            };

            ret = &ret + &dir;
        }
        Ok(ret)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
enum Color {
    WHITE,
    BLACK,
}

struct Board {
    state: HashMap<HexVector, Color>,
}

impl Board {
    fn color_of(&self, hv: &HexVector) -> Color {
        *self.state.get(hv).unwrap_or(&Color::WHITE)
    }

    fn neighbor_black_count(&self, hv: &HexVector) -> u32 {
        hv.neighbors()
            .filter(|neighbor| self.color_of(&neighbor)==Color::BLACK)
            .count() as u32
    }

    fn round(&mut self) {
        let to_check: HashSet<HexVector> = self.state.iter()
            .filter(|(_, &color)| color == Color::BLACK)
            .flat_map(|(hv, _)|
                hv.neighbors()
                    .map(|neighbor| neighbor)
                    .chain(iter::once(hv.clone()))
            )
            .collect();

        let changes: Vec<_> = to_check.iter()
            .map(|hv| (hv, self.color_of(hv), self.neighbor_black_count(hv)))
            .filter_map(|(hv, current_color, blacks)| {
                if current_color == Color::WHITE && blacks == 2 {
                    Some((hv, Color::BLACK))
                } else if current_color == Color::BLACK && (blacks == 0 || blacks > 2) {
                    Some((hv, Color::WHITE))
                } else {
                    None
                }
            })
            .collect();
        for (hv, color) in changes {
            self.state.insert(hv.clone(), color);
        }
    }

    fn blacks(&self) -> u32 {
        self.state.values()
            .filter(|&c| *c == Color::BLACK)
            .count() as u32
    }
}

fn main() -> Result<()> {
    let input: Vec<HexVector> = INPUT.lines().map(|l| l.parse().unwrap()).collect();

    let tile_flips = input.iter()
        .fold(HashMap::new(), |mut acc, v| {
            let v = acc.entry(v).or_insert(0);
            *v += 1;
            acc
        });

    let blacks = tile_flips.iter()
        .filter(|(_, &flips)| flips % 2 == 1)
        .count();

    dbg!(blacks);

    let mut board = Board{
        state: tile_flips.iter()
            .map(|(&k, v)| (k.clone(), if v % 2 == 1 { Color::BLACK } else { Color::WHITE}))
            .collect()
    };

    for _ in 0..100 {
        board.round();
    }

    dbg!(&board.blacks());

    Ok(())
}

const INPUT: &str = r#"wseseseswsesesewnesesesesewneseseswnene
seswswsenenweseneesweenwswneswsenwsenw
nenwneneneeneneneneseeenene
swwenwseswseswwnweswswswnwwswesww
nwenenwnwwswnwsewneneswnwswnwwwwsenw
seswseseswsesweseswswswnw
seneesweneesweweeseeseeeenese
nwswnwenwnwnwnwseneswnenwneswswsenenwnenw
wsewwwwwwnenwwwwwneswww
wneneneneenenewneneenesenenenee
sesenweswswneswneswwswnwnwseswseswswnesw
eenwneswseswnwneseneneswwesenwswnwwnese
swnwwenenenwnwnwnwwswnwnwnwnwnwnwneswsw
neeeweswneesweeseeeenweswswwnenw
swswseswnwswseseseneneswsw
eeswnwwnwsenenesenwnwnenwenwnwnenwwnwsw
wseeseswsesewenwswswnweseseseseseswswne
newnewneneneeneswseeneeneeneneese
wenweeswseeeeweseeswneeenwene
swneneswswswnwseseseseswneswswsesesesese
weweewwswwwnw
esesesesewesesenweseseeneswswenesee
eswneswswnwswenewnenwneeeswneswnene
esenenesewseseseesewsesese
wswseneseseeewseswnwneeseneewenee
swseswseewswweeswnwewswenwnewsese
seswseseswwneswswswseswswnwswnwnewnene
sweeesenenenewnenenwneneeneeenewsw
wnwwwwseswnwwwswneswswesewswsenwsww
swswseswswnwneesewswseswneswswsw
seenwseswseesenwseeseseseseseseswswne
swswswnewswswswswswswswsenewwwnewse
swswsewneneswseswnewseswneswneseswsese
wwswnwewswsesenweneeeenwwneee
nwnenenwnwnenenenwnewnwnwnwswe
wwsenwsewwweewwswwswwwnenew
wnenwewnenenenweneenwnwswsw
weeewsweeeeneeeenwneneneneesw
sewnwnwwnwwnwwneweeswnwsenwswnw
swnenenenwswnweswnenenenewnenenene
nwswnwnwsenwesenwnwnwnenwsenenwnwnwwnwsew
nweeeenenenewsweeeneeeeesewese
neneneneneenenwneneeneewnesw
nenwnenwnwnwswnwnwnwnwnwnwnw
nenwswnwnwnwwswnwnwwwnwenwsenwneswnwne
wseneseseseesweseseeenenwenewsese
senwwsesenwenwenenwewseswnweswsese
eweseeseswenwseseeseesenwnwnenwee
wwnwnwwenwnesewwnwnwswwnweeswww
neeneeeneeenenenenwenese
nesenwnwneenwnenwwswnwseswswwwnwene
nesweeswwseswsesewnesewswnw
nwnwewnwnewswnwswwnww
enwnesenwsweseeeewneseeee
seneswwsesesenesesesesesesesenwsesesese
seweswsenesweswswnwswswswwswswswswsw
swswswswswwswneenwswswswswswsewswsene
nenwenewnwnwneneswneneneneneneswenenesene
esenwsesesesesesesesesese
nenwnenwnwnwnenenwnwnenweswnene
swwwwwnewnwnwseewnww
nesenenenenwnewnenwswnweneesenew
wwwwwwwwsewswwswwnw
sesweswswswenwnwswswswswwseswswnwswswswnw
sewnwswswewesee
swnwswswsewswswswneseswswswsw
seswenwnwseeswwnwneseesenweseseesenw
ewesenwseseseeseeseseswsewwwsesw
neeswnenwseenwnenenenwewwnenwnenwnw
eswweeeneneeswenwneenwsewseswse
wwswseseeeseewnwseesesesenwsenese
nwswnwnwenwweswseneswswneswsenenenw
nwwweswewswwnwswswswewwwnwwseee
seseeswsewnwnenweneneenwse
swswwwswsewnwswswww
nwseweseesewsesesewneseseswesese
wwwwnewwwwnwnenwwswwsw
nwenenwseneneswnesenenwnwsewnwswwnwswnwnw
wswneswswswenwseswnwswsesweswswwsesenwne
neesenwnenwseseesweswnwenwneeewe
wneswwwwswwwswweeswweswswwww
wnwewwwwwewwswwwwwwwew
seewwswwwswwneswswwneswnw
swnwswnwsesenwnweswswwseneeewswswsw
swenenwneswwnenweswswnenweeeeeseee
wnwnwneenwswswsenwseenwnwnenwwnwwnene
neneeneswnwneeseewneeweneneneenee
seesenwnwsenesesweseseseswseeneswnwwnw
nweeseseneeenewneeeeeswenwneee
swnenesenwswnenwnwewweswswnwwnenese
senwnewwsenwnwnwnenwnwnwnwnwneeesenwnw
swseenwswneseeseeseseseeewnesenee
sewwswwsenesewnenwewwneeswswee
swneneeeswneneeenenwnene
swwneswswwswseswswswsewwwwnww
nenwnenenenenenesenwsweneneswnenenwnenw
nweswneswseeeeeeswneeeeeneew
swneseneenweeeenwsweseenwsw
nwwswswwenweswwswswwswswswneewwsw
nesweeewsenewsenwnenenwneeneneenenene
swswwswnwwneeswweswswwswswesewwswe
seswseseseeenenweenewwsewseseesese
wswweswwseswwwnw
wsenwsesesenwnenenenwnwswwwnwnenwswwne
swswsweseswswswswswnwswsweswnwswneswswsw
wswwwnwwwwwe
seneseswseseeswnewswswsenwswsesesesese
neesenenesenwenweenwsewneew
wswwswswwswnwswwewswewwnwswesw
swsewsweseneseseneswnese
seseseneswesesesesesesese
newwnesesewnwnenewneneneneswneneswenese
eseswneeenewsenwnewee
nwswswswneswwsweswsw
nwwswwswswsewnwswweewwswswswwswnese
wnewseneeeesweseeewseseseseseneese
swenewnwnweeneneneneswnewsenwnwnenw
nweweeeeswneweseneneneeeswnene
eswneseeeesenweewseweseneesee
nwwswwneneenwsenenwnweneneneeneswe
eweswneneeeseeswenwsenenwnesesewenw
nwseneeswneesesenenwnwneneeswnenwnenwse
swswnwsweeswnesenwswswswswswswswsenwse
nenwnwenwnwswneswnwsenwnwnwenwnwswnwsesw
nwseneenwnwnenwwnwnenwsw
nwnesewswswswsweswwswneswnweswe
enwswwwnewnewwswwwwnwswwwseww
nenwnenenwwneweneseswenwnenwneeswswne
nenwnenwswwneneneneenwswnwneswneneneenesw
neseseneswnewnenesesenenwnwneswsenwnwnw
swenenenenenenesenewnene
nwnwnwnwnwnwnwnwnwenwnwnwswnw
neseneswwnwneswnwsewseseeswenwneswswsw
wwwwneenwnwseswenenwnenwwwnwsesw
wwnewswwwnewswwsewwww
sesesewswsweseswseswse
seswewneenwnenwnwnenwnwnwnwneneswswnwnenw
neeseenwseesweneseweeseeesenwsw
swneneneenewswneneewnenwnwwnwswswnesese
swnesenwnwnwnewnwnenenenwnenenw
wwwnewneeswsenwnwswwnwnwenwnwnewnw
eeeeeeneweeseeeeee
neweeswsenenwseeeeeneneneeneeneswnw
nwnwwenwnwseenwnwenwwnwswwnwswewne
wneenewenweenwseseseesesenewww
eewneeneneeneeenene
swswsenwweenwswswswswswswswsw
nwneneseeenwnwnwswnwwswnwse
swenwnwenwswwnwnwswneenwnwnwnenwnwnwse
wsewswwnenwneseswsenwsww
seeeenwseseneeeeseeeswesew
weenwnewnenwseenwnwnenwnenwnenwwne
sweeeswesenwwnwswwneswnwwewenenw
eeneeenwesewweeneeeswewee
sweseswseswneseseswswsewsesw
wneswseseseseswseseseswswwswsesee
swswswswswseswswwwseeseenewnewsw
eeswnwnwnesewswneseneswsesweewswsee
eseweeneseseseee
weswswnwswsesewswsewenwneneswswenwnw
seseenwseeswneswseenwseseewee
eeneneeswsweseswewwneenwe
neseewnwwesewnwwwswwsewnesenew
eneswnenwewneseneneenenweneneseene
sesenesesesesesenewswsesewseseseseseew
nenenwnenwneeneneneneenwwswneeswnwesw
wwewwsewewwsewwnwwwnwnewww
nweneewnwseneseseeeeswsenweeenew
nwnwwseswseeseseseneenwewswnwseswse
neewsweseeeweswnwsenenwnwwwne
senenwnewseseeswsweseseswswnwwseesesenw
neswswnwnenenwnwsewwnwneswswenwsewnwnenw
wwswwenwnewseneeswsewewnw
seswsewseseneseswneesesenwseseswsesesene
ewwsenenwwseswswsenwnewweenwwenww
nenweenwneenwseneneewwsweeesewe
wswnwwwewwwsewnwsewwsewnwnwse
nwnenwnwenwnwenwnwnesweneswnwnwnwnwswnenw
seeneswwesesesewswsenwwwneewnwne
swwnwswseeswwswswenwswsweswswswwwnw
seswnwewswseswwswwseenesw
eneneseneneneeeneswneneeewnweseswne
enwwneeneeeeesw
sesesenwsesenweesweseeseseneseseswe
nwnwnwnenwenenenenww
nwnwwnwswenwnwnwwnwnwnwwesewnwwnwsw
wwneesenwwsenweswwsewsewwnenww
swswseswswneneswswswswwsweseseswnwswse
wseenwwwwnwnwnenwnwwnwesewsenwnww
wseneseswswwwsewswswwnewsweneswswnww
nwsewnwwnenwnesewsewwnwwswnwnwswnw
senwenesenenenesweenewnewnenenenenw
swnwswweewnwwwnwswseswewewwsww
wsewswnewsenewsewwwswwwwwwnw
nwsenwnenenenenwnwnwnenwnw
nenenwnenwnwnwnwnwenwwnesenwswnene
nwwweewenweeswesw
wnwnwnwsenwswnenwweswwnwnenwwwnwww
seseneenwseneneswwneseneeeweneneenw
swswwswswwswswswswwwesww
nenesenewnwsenenenenwwnenwenenenenww
wnwneseswswswswswnenewswenwswwsesesw
wwwwnwseswwswwsww
nwseenenwswweeseseneseeesewswsesene
nenewsenewwneesweneesewwnwe
wwswwewwwwseswwwweswnweesw
nesweewswsweneeneneenwne
wseseswnwesenwnewseneswenwneswseesw
nwnwwswsenweeswswnwseswsweswse
nenwnwnwnwswneneenenwswswsenwnwnenwnwsenw
wswnewwnwswwwewnwnwsenwnw
nwswneneseeseneeswwne
nwseseswswneswnewsenwseneseswseeseswsenw
enwnwnwswnwnwseneswnwswneswenwwwnesee
wnwnwwswnwnwnenwwwnwnwnwnw
nenwewnesewwswwswneswwseewnwww
nwswsweewswsesesesweswnewnewneesesw
nenwsewwwswwwwswwseswww
nwnwseneneneneneswnwnwnwneswewneneewne
weswsweseseneswsenwsenwseneneesesenwsesw
nwnwswwnwnwnwwnwwnwenw
swnwnwnwenwnwneswnwenwsenwnwnwnwnwnenw
neswnenwneenwswnesenwneesweneswwnene
neeeneneeeeeeesewweeesw
eswneneneseswseswswsesenwnwseswseseswse
seeeeneseseseseswenwnweseesw
wewewnenwnwnesweenwnwwsewswnew
seseweseseseeseneesesewswenewse
wnesewswwswwswwwwneewwwwswsewnw
nwweseswnwwsweseseeneneseeesewsese
nwswneeseewseswneeswnwwwnw
seseswwneesenwneseseeenweseseseesesw
enwnenenenweneneneneneswswnenenesene
eneswseeneseweeeeneenweenwene
swwwwwwwwswnwe
nwsesesesesesewnewseeeswsesee
nwseeseeseseeswsenweneseesweee
nweswwneswnwwnwswweswnwnwnwnwnwnewe
neneswseswenenwnweseeswseseswwnwswswne
wnwnwwwwneswewnwwwseswnenwwww
wnwenwsewwwneseswnwesweeeewse
seeseswnweseseeswswwnwswnwnwwswsee
swswswewsewswseswnwnwwswswswswnewswse
eswseneswnwsweneenw
enwnwswnwwsewwenwnwnwsenwwnwnwnwnw
eeeenesewseeeeeeseweneswwee
eswsweeneneneeeneseneneneewswnwnwe
nwwseseswsenwnwenenewwneneneenewnwnw
nwnwnwnwnwnenwnwnwnwnwnwnwnesw
wneeneswewneneenesenwnwswne
nenweswneesweeeneee
neseneswneneneseenenewenenwneneenwse
swnwswsweneswswswwnwswswswsewseseswwnew
swenwwnesenweewww
swswwwwnwswwwnwnwwswwwseewswe
seswseseswsesesesesenwsese
nenwsewnesenwnenenwnenenesenenenewseswne
nenwnwnwswnenweneenwnenwneswswswnenwesw
wswseswewneseseesenenenwnewswseseswswsw
nesewseseseseseseeseseseseseswenwswnwse
senweswswswseswseswseweeswswwse
wwweesewnwnwswswswswswwwnenwwww
nwswseneenwswswwnwsweseswswswwswswsw
weswneneswneenenesenesewnenenwnenwsesw
neneenenenenenenenwswneneneneeswne
eeenwseseeeeeswseweeenwneeee
swswswswswswwsweswswnwswwenwswswsw
senwnwswswswswewnesenewwswnwwwswsw
nenenwneswnenwnwwnesenenewnenwnenwesese
wwsewnewswnweswsesenwnwnwnwsweswe
nwnenwnwsenwnwnwnwnwwnwseswwnwwnenwewnw
seswswnenweeswnwnwnwswswswneswse
nenesenweeseeneneswneenweweeswene
enesweneeneewneswwnenewneenwnw
swseseswsesenweswnwnwnesenw
nwsenwwnwnwneenwwwnwwwwwnww
seeseneswseswswswsesewswswseswsenewwse
swnenenenwewnenwsenwsenwnwnwnwnw
ewwwswnenwswnwwsenwewneenwwnwwse
swneenweeeneenweeeeeswnweeswnwse
sweeeeseenweeeeeenwwee
weswwwwwnesewwwswwswswwnw
swswswswswswnwswswswswswswswe
wnwnwnenwswnwsenwsenwwnwnwnwwnenwswe
swwwswwwnwnwnweswnwwwenwnwenewne
eenwesesewseenw
neeeseseeenwenwsweeseeeeewe
swnesenwnesenwnwnenwnenewswseenenenwnw
enwwneenwneswnwsenenwnwwnwnwnwnwnw
swswnwnweenwswwneneeswnwnwnewnwwswsene
neswwswsenenwwnwneesesenwnwswweswnee
seswsesesenewneswsese
seneswnenwswswnwneeneswnenwwnesenenww
sesesesesenewewseeseene
eseeewneeneeeeweeneswnweneneswe
neswswswswswseeswswnwwswsweswwsw
wnenwswsewenwnwswwnwenwswwnwnwee
seseneswsewswseseswnesesesesesenwswsese
enewswswnwsewnweswswneswwewwwneswse
seseseswnwnwnwnwnwnwnenwnenwnwnewwnwnwnw
neswnenwwneswswswew
seseenwneneneweseeenwnenewswwwneew
swswswneswswwnwswsweeswswseswseswnwseswne
nenenwwneswswenwnwnenenwnenewnesenenw
enwwneneeeneenenenewsesenewsenenenesw
swseseseseseseseseswnwsesese
swnwnwewsesewenesenwswew
eesenwnwenesweswweeswswsenwnwse
nwwneseseseneneswwswseswswsewswneswsenw
sweseswnwswnwswswswswswswsenwswswesweswsw
seneswneneenwnewnwwseswnewene
wnwnesewnenenwseswnwnenwneenenenenenee
seseeseneswswseseseeenwsenenwsesesee
"#;
const TEST: &str = r#"sesenwnenenewseeswwswswwnenewsewsw
neeenesenwnwwswnenewnwwsewnenwseswesw
seswneswswsenwwnwse
nwnwneseeswswnenewneswwnewseswneseene
swweswneswnenwsewnwneneseenw
eesenwseswswnenwswnwnwsewwnwsene
sewnenenenesenwsewnenwwwse
wenwwweseeeweswwwnwwe
wsweesenenewnwwnwsenewsenwwsesesenwne
neeswseenwwswnwswswnw
nenwswwsewswnenenewsenwsenwnesesenew
enewnwewneswsewnwswenweswnenwsenwsw
sweneswneswneneenwnewenewwneswswnese
swwesenesewenwneswnwwneseswwne
enesenwswwswneneswsenwnewswseenwsese
wnwnesenesenenwwnenwsewesewsesesew
nenewswnwewswnenesenwnesewesw
eneswnwswnwsenenwnwnwwseeswneewsenese
neswnwewnwnwseenwseesewsenwsweewe
wseweeenwnesenwwwswnew"#;
