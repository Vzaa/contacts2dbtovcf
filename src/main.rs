extern crate rusqlite;

use rusqlite::Connection;

#[derive(Debug)]
struct Contact {
    id: i32,
    name: String,
    phone_nums: Vec<String>,
}

fn main() {
    let db_filename = std::env::args().nth(1).unwrap_or("contacts2.db".to_owned());

    let conn = Connection::open(db_filename).unwrap();

    let contact_query = "SELECT _id,display_name FROM raw_contacts";
    let mut stmt = conn.prepare(contact_query).unwrap();

    let contact_iter = stmt.query_map(&[], |row| {
        let id = row.get(0);
        let name = row.get(1);
        let mut phone_nums = Vec::new();

        let num_query = format!("SELECT normalized_number FROM phone_lookup WHERE raw_contact_id={}", id);

        let mut stmt2 = conn.prepare(&num_query).unwrap();
        let phone_num_iter = stmt2.query_map(&[], |row| row.get(0)).unwrap();

        // Replace 00 in numbers with +
        let phone_num_mapped = phone_num_iter
            .map(|x| x.unwrap())
            .map(|x: String| {
                if x.starts_with("00") {
                    x.replacen("00", "+", 1)
                } else {
                    x
                }
            }
        );

        for phone_num in phone_num_mapped {
            // Don't add if a previous entry ends with this
            if phone_nums.iter().any(|x: &String| x.ends_with(&phone_num)) {
                continue;
            }

            // Remove previous entries that end with this
            phone_nums.retain(|x: &String| !phone_num.ends_with(x));

            phone_nums.push(phone_num);
        }

        Contact { id, name, phone_nums }
    }).unwrap();

    for c in contact_iter {
        let con = c.unwrap();

        if con.phone_nums.len() > 0 {
            println!("BEGIN:VCARD\nVERSION:3.0\nFN:{}", con.name);
            for p in con.phone_nums[1..].iter() {
                println!("TYPE=VOICE:{}", p);
            }
            println!("TEL;TYPE=VOICE;TYPE=PREF:{}\nEND:VCARD", con.phone_nums[0]);
        }
    }
}
