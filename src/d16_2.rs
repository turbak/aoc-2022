#![feature(drain_filter)]

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
};

#[derive(Debug, PartialEq, Eq, Hash)]
struct Valve<'a> {
    name: &'a str,
    flow_rate: usize,
}

fn main() {
    let split_lines = include_str!("../inputs/d16").lines().map(|l| l.split("; "));

    let mut valves = HashMap::new();
    let mut adjecency_matrix_str = HashMap::new();
    for mut line in split_lines {
        let first = line.next().unwrap();
        let second = line.next().unwrap();
        let valve_split = &mut first.split(" ");

        let valve = Valve {
            name: valve_split.skip(1).next().unwrap(),
            flow_rate: valve_split.skip(2).next().unwrap()["rate=".len()..]
                .parse()
                .unwrap(),
        };

        adjecency_matrix_str.insert(
            valve.name,
            second
                .split(" ")
                .skip(4)
                .map(|s| s.trim_end_matches(","))
                .collect::<Vec<&str>>(),
        );
        valves.insert(valve.name, valve);
    }

    let adjacency_matrix: HashMap<&Valve, Vec<&Valve>> = adjecency_matrix_str
        .iter()
        .map(|(v, names)| {
            let valve = valves.get(v).unwrap();
            let neighbours = names
                .iter()
                .map(|name| valves.get(name).unwrap())
                .collect::<Vec<&Valve>>();
            return (valve, neighbours);
        })
        .collect();

    let mut distances = HashMap::<&Valve, HashMap<&Valve, usize>>::new();
    valves.iter().for_each(|(_, v)| {
        distances.insert(v, dijkstra(v, &adjacency_matrix));
    });

    let distances_from_start: HashMap<&Valve, usize> = distances
        .get(valves.get("AA").unwrap())
        .unwrap()
        .iter()
        .filter_map(|(v, d)| {
            if v.flow_rate > 0 {
                return Some((*v, *d));
            }
            return None;
        })
        .collect();

    distances = distances
        .iter()
        .filter_map(|(v, d)| {
            if v.flow_rate <= 0 {
                return None;
            }
            let filtered_map = d
                .iter()
                .filter_map(|(valve, distance)| {
                    if valve.flow_rate > 0 {
                        return Some((*valve, *distance));
                    }
                    None
                })
                .collect::<HashMap<&Valve, usize>>();
            return Some((*v, filtered_map));
        })
        .collect();

    println!("from_start: {:#?}", distances_from_start);
    println!("distances: {:#?}", distances);

    let mut max_flow = 0;
    for (neighbour1, distance1) in distances_from_start.iter() {
        for (neighbour2, distance2) in distances_from_start.iter() {
            if neighbour1 == neighbour2 {
                continue;
            }

            max_flow = max_flow.max(find_max_flow(
                vec![
                    ValveOpener {
                        current_valve: neighbour1,
                        time_spent: distance1 + 1,
                    },
                    ValveOpener {
                        current_valve: neighbour2,
                        time_spent: distance2 + 1,
                    },
                ],
                &distances,
                HashSet::new(),
            ));
        }
    }

    println!("solved: {}", max_flow)
}

struct ValveOpener<'a> {
    current_valve: &'a Valve<'a>,
    time_spent: usize,
}

const MAX_MINUTES: usize = 26;

fn find_max_flow<'a>(
    mut current: Vec<ValveOpener<'a>>,
    distances: &HashMap<&'a Valve, HashMap<&'a Valve, usize>>,
    mut open_valves: HashSet<&'a Valve<'a>>,
) -> usize {
    if current.iter().all(|v| v.time_spent > MAX_MINUTES) {
        return 0;
    }

    current = current
        .drain_filter(|v| v.time_spent <= MAX_MINUTES)
        .collect();

    for v in current.iter() {
        if !open_valves.insert(v.current_valve) {
            return 0;
        }
    }

    if open_valves.len() == distances.len() {
        return current
            .iter()
            .map(|v| (MAX_MINUTES - v.time_spent) * v.current_valve.flow_rate)
            .sum();
    }

    let mut max_flow = 0;
    if current.len() == 1 {
        let opener = current.first().unwrap();
        for (neighbour, distance) in distances.get(opener.current_valve).unwrap() {
            if open_valves.contains(neighbour) {
                continue;
            }

            max_flow = max_flow.max(find_max_flow(
                vec![ValveOpener {
                    current_valve: neighbour,
                    time_spent: opener.time_spent + distance + 1,
                }],
                distances,
                open_valves.clone(),
            ))
        }
    } else {
        let opener1 = current.get(0).unwrap();
        let opener2 = current.get(1).unwrap();
        for (neighbour1, distance1) in distances.get(opener1.current_valve).unwrap() {
            if open_valves.contains(neighbour1) || distance1 + opener1.time_spent + 1 > MAX_MINUTES {
                continue;
            }
            for (neighbour2, distance2) in distances.get(opener2.current_valve).unwrap() {
                if neighbour1 == neighbour2 || open_valves.contains(neighbour2) {
                    continue;
                }

                max_flow = max_flow.max(find_max_flow(
                    vec![
                        ValveOpener {
                            current_valve: neighbour1,
                            time_spent: opener1.time_spent + distance1 + 1,
                        },
                        ValveOpener {
                            current_valve: neighbour2,
                            time_spent: opener2.time_spent + distance2 + 1,
                        },
                    ],
                    distances,
                    open_valves.clone(),
                ));
            }
        }
    }

    max_flow
        + current
            .iter()
            .map(|v| (MAX_MINUTES - v.time_spent) * v.current_valve.flow_rate)
            .sum::<usize>()
}

#[derive(Debug)]
struct Visit<V> {
    vertex: V,
    distance: usize,
}

impl<V> Ord for Visit<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl<V> PartialOrd for Visit<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> PartialEq for Visit<V> {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl<V> Eq for Visit<V> {}

fn dijkstra<'a>(
    start: &'a Valve,
    adjacency_list: &HashMap<&'a Valve, Vec<&'a Valve>>,
) -> HashMap<&'a Valve<'a>, usize> {
    let mut unexplored = BinaryHeap::new();
    let mut distances = HashMap::<&Valve, usize>::new();
    let mut visited = HashSet::new();
    let mut path = HashMap::new();

    distances.insert(start, 0);
    unexplored.push(Visit {
        vertex: start,
        distance: 0,
    });

    while let Some(Visit { vertex, distance }) = unexplored.pop() {
        if !visited.insert(vertex) {
            continue;
        }

        for neighbour in adjacency_list.get(vertex).unwrap().iter() {
            let new_distance = distance + 1;
            let is_shorter = distances
                .get(*neighbour)
                .map_or(true, |&current| new_distance < current);

            if is_shorter {
                distances.insert(*neighbour, new_distance);
                unexplored.push(Visit {
                    vertex: *neighbour,
                    distance: new_distance,
                });
                path.insert(*neighbour, vertex);
            }
        }
    }

    return distances;
}
