use crate::config::Committee;
use crate::core::RoundNumber;
use crypto::PublicKey;

pub type LeaderElector = RRLeaderElector;

pub struct RRLeaderElector {
    committee: Committee,
}

impl RRLeaderElector {
    pub fn new(committee: Committee) -> Self {
        Self { committee }
    }

    pub fn get_leader(&self, round: RoundNumber) -> PublicKey {
        let mut keys: Vec<_> = self.committee.authorities.keys().cloned().collect();
        keys.sort();
        keys[round as usize % self.committee.size()]
    }
}

// pub struct  DummyLeaderElector {
//     committee: Committee,
// }

// impl DummyLeaderElector {
//     pub fn new(committee: Committee) -> Self {
//         Self { 
//             committee,
//          }
//     }

//     pub fn get_leader(&self, round: RoundNumber) -> PublicKey {
//         let mut keys: Vec<_> = self.committee.authorities.keys().cloned().collect();
//         keys.sort();
//         keys[round as usize % self.committee.size()]
//         // for i in 0..keys.len(){
//         //     if committee.authorities[keys[i]]["id"] == 0{
//         //         break keys[i];
//         //     }
//         // }
//     }
// }