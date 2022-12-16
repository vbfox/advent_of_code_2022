use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::Add,
};

fn reconstruct_path<T>(came_from: &HashMap<T, T>, current: &T) -> Vec<T>
where
    T: Eq + Hash + Clone,
{
    let mut total_path = Vec::new();
    let mut current = current;
    while came_from.contains_key(current) {
        current = &came_from[current];
        total_path.push(current.clone());
    }

    total_path.reverse();
    total_path
}

/// A* finds a path from start to goal.
///
/// # Arguments
///
/// * `start` - The start node
/// * `goal` - The goal node
/// * `heuristic` - The heuristic function. Estimates the cost to reach goal from the passed-in node.
/// * `neighbors` - The function to get the neighbors of a node
/// * `neighbor_distance` - The function to get the distance between the current node and a neighbor
#[allow(clippy::needless_pass_by_value)]
pub fn a_start<TNode, FHeuristic, FNeighbors, FDistance>(
    start: TNode,
    goal: TNode,
    heuristic: FHeuristic,
    neighbors: FNeighbors,
    neighbor_distance: FDistance,
) -> Option<Vec<TNode>>
where
    FHeuristic: Fn(&TNode) -> i32,
    FNeighbors: Fn(&TNode) -> Vec<TNode>,
    FDistance: Fn(&TNode, &TNode) -> i32,
    TNode: Eq + Hash + Clone + Ord,
{
    let mut open_set = BTreeSet::new();
    open_set.insert(start.clone());

    let mut came_from = HashMap::<TNode, TNode>::new();

    let mut g_score = HashMap::<TNode, i32>::new();
    g_score.insert(start.clone(), 0);

    let mut f_score = HashMap::<TNode, i32>::new();
    f_score.insert(start.clone(), heuristic(&start));

    while !open_set.is_empty() {
        let current = open_set
            .iter()
            .filter_map(|p| f_score.get(p).map(|s| (p, s)))
            .min_by_key(|(_, s)| *s)
            .unwrap()
            .0
            .clone();

        if current == goal {
            return Some(reconstruct_path(&came_from, &current));
        }

        open_set.remove(&current);

        for neighbor in neighbors(&current) {
            let neighbor_distance_value = neighbor_distance(&current, &neighbor);
            let tentative_g_score = g_score[&current] + neighbor_distance_value;
            let neighbor_score = g_score.get(&neighbor);
            if neighbor_score.is_none() || tentative_g_score < *neighbor_score.unwrap() {
                came_from.insert(neighbor.clone(), current.clone());
                g_score.insert(neighbor.clone(), tentative_g_score);
                f_score.insert(neighbor.clone(), tentative_g_score + heuristic(&neighbor));

                if !open_set.contains(&neighbor) {
                    open_set.insert(neighbor.clone());
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct DijkstraResult<TVertex, TDistance> {
    /// The shortest path from the start to the end, or at all if no end in specified
    pub distance_to_end: Option<TDistance>,

    /// Distances from the start to all nodes.
    pub distances: HashMap<TVertex, TDistance>,
}

// https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm
#[allow(clippy::needless_pass_by_value)]
pub fn dijkstra<TVertex, TDistance, FNeighbors, FDistance>(
    start: TVertex,
    goal: Option<TVertex>,
    neighbors: FNeighbors,
    neighbor_distance: FDistance,
    all_nodes: Vec<TVertex>,
) -> DijkstraResult<TVertex, TDistance>
where
    FNeighbors: Fn(&TVertex) -> Vec<TVertex>,
    FDistance: Fn(&TVertex, &TVertex) -> TDistance,
    TVertex: Eq + Hash + Clone,
    TDistance: Default + Copy + Ord + Add<Output = TDistance>,
{
    // Mark all nodes unvisited. Create a set of all the unvisited nodes called the unvisited set.
    let mut unvisited = HashSet::<TVertex>::new();
    unvisited.reserve(all_nodes.len());
    for node in all_nodes {
        unvisited.insert(node.clone());
    }

    // Assign to every node a tentative distance value: set it to zero for our initial node and to infinity
    // for all other nodes.
    let mut tentative_distances = HashMap::<TVertex, TDistance>::new();
    tentative_distances.reserve(unvisited.capacity());
    tentative_distances.insert(start.clone(), TDistance::default());

    // Set the initial node as current
    let mut current = start;

    loop {
        let tentative_distance = *tentative_distances
            .get(&current)
            .expect("current node not in tentative distances");

        // For the current node, consider all of its unvisited neighbors and calculate their tentative distances
        // through the current node.
        for neighbor in neighbors(&current).iter().filter(|p| unvisited.contains(p)) {
            let new_tentative_distance = tentative_distance + neighbor_distance(&current, neighbor);
            let current_tentative_distance = tentative_distances.get(neighbor);

            // Compare the newly calculated tentative distance to the one currently assigned to the neighbor and
            // assign it the smaller one.
            if current_tentative_distance.is_none()
                || new_tentative_distance < *current_tentative_distance.unwrap()
            {
                tentative_distances.insert(neighbor.clone(), new_tentative_distance);
            }
        }

        // When we are done considering all of the unvisited neighbors of the current node, mark the current node
        // as visited and remove it from the unvisited set
        unvisited.remove(&current);

        // If the destination node has been marked visited
        if let Some(ref end) = goal && &current == end {
            // We are done
            let distance_to_end = tentative_distances.get(end).expect("end node not in tentative distances");
            return  DijkstraResult {
                distance_to_end: Some(*distance_to_end),
                distances: tentative_distances,
            };
        }

        let next = tentative_distances
            .iter()
            .filter(|(p, _)| unvisited.contains(p))
            .min_by_key(|(_, d)| *d)
            .map(|(p, _)| p)
            .cloned();

        match next {
            // Otherwise, select the unvisited node that is marked with the smallest tentative distance, set it as
            // the new current node
            Some(next) => current = next,
            // if the smallest tentative distance among the nodes in the unvisited set is infinity (when planning
            // a complete traversal; occurs when there is no connection between the initial node and remaining
            // unvisited nodes)
            None => {
                return DijkstraResult {
                    distance_to_end: None,
                    distances: tentative_distances,
                }
            }
        }
    }
}
