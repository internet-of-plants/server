// Free: All Past Day Events + One User + One Device
// 5 dolars per Device: Past Week Grouped By Hour + Before that Grouped By Day + All Past Day Events
// Enterprise: 10 dolars per device: ETL + All Events + Support

use serde::{Deserialize, Serialize};

const DEFAULT_CAPABILITIES: &[Capability] = [Capability::AllPastDay, Capability::MultipleUsers, Capability::MultipleDevices]

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
enum Capability {
    AllPastDay,
    PastWeekGroupedByHour,
    RestGroupedByDay,
    AllEvents,
 
    MultipleUsers,
    MultipleDevices,
}
