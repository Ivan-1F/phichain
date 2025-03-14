use crate::lifetime::{find_lifetime, merge_lifetimes};
use itertools::Itertools;
use phichain_chart::serialization::{PhichainChart, SerializedLine};

// TODO: wip
pub fn reuse_lines(chart: PhichainChart) -> PhichainChart {
    let mut groups: Vec<Vec<SerializedLine>> = vec![];
    for (a, b) in chart.lines.iter().tuple_combinations() {
        let a_lifetime = find_lifetime(a);
        let b_lifetime = find_lifetime(b);
        for group in &mut groups {
            let lifetime =
                merge_lifetimes(group.iter().map(|l| find_lifetime(l).clone()).collect());

            if !lifetime.overlaps(&a_lifetime) {
                group.push(a.clone());
                break;
            }

            if !lifetime.overlaps(&b_lifetime) {
                group.push(b.clone());
                break;
            }
        }

        if !a_lifetime.overlaps(&b_lifetime) {
            groups.push(vec![a.clone(), b.clone()]);
        } else {
            groups.push(vec![a.clone()]);
            groups.push(vec![b.clone()]);
        }
    }

    chart
}
