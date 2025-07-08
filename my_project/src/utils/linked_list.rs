#[derive(Debug, Clone)]
pub struct VariationNode {
    pub variation: String,
    pub next_variation: Option<Box<VariationNode>>,
}

#[derive(Debug, Clone)]
pub struct IdentityNode {
    pub first_name: String,
    pub last_name: String,
    pub father_name: String,
    pub grandfather_name: String,
    pub mother_last_name: String,
    pub mother_name: String,
    pub dob: Option<(u32, u32, u32)>,
    pub sex: u8,
    pub place_of_birth: String,

    pub first_name_variations: Option<Box<VariationNode>>,
    pub last_name_variations: Option<Box<VariationNode>>,
    pub father_name_variations: Option<Box<VariationNode>>,
    pub grandfather_name_variations: Option<Box<VariationNode>>,
    pub mother_last_name_variations: Option<Box<VariationNode>>,
    pub mother_name_variations: Option<Box<VariationNode>>,

    pub next_identity: Option<Box<IdentityNode>>,
}

// Insert a variation into a sorted linked list (no duplicates)
pub fn insert_variation(head: &mut Option<Box<VariationNode>>, variation: &str) {
    let mut current = head;

    loop {
        let needs_insert = match current {
            Some(node) if node.variation == variation => return,
            Some(node) if node.variation.as_str() > variation => true,
            Some(node) => {
                current = &mut node.next_variation;
                continue;
            }
            None => true,
        };

        if needs_insert {
            let next = current.take();
            let new_node = Box::new(VariationNode {
                variation: variation.to_string(),
                next_variation: next,
            });
            *current = Some(new_node);
            return;
        }
    }
}

pub fn insert_identity(
    head: &mut Option<Box<IdentityNode>>,
    first_name: &str,
    last_name: &str,
    father_name: &str,
    grandfather_name: &str,
    mother_last_name: &str,
    mother_name: &str,
    dob: Option<(u32, u32, u32)>,
    sex: u8,
    place_of_birth: &str,
    first_name_var: &str,
    last_name_var: &str,
    father_name_var: &str,
    grandfather_name_var: &str,
    mother_last_name_var: &str,
    mother_name_var: &str,
) {
    let new_key = format!(
        "{}{}{}{}{}{}",
        first_name, last_name, father_name, grandfather_name, mother_last_name, mother_name
    );

    let mut cursor = head;

    loop {
        // Separate scope for borrowing
        let insert_here = match cursor {
            Some(node) => {
                let node_key = format!(
                    "{}{}{}{}{}{}",
                    node.first_name,
                    node.last_name,
                    node.father_name,
                    node.grandfather_name,
                    node.mother_last_name,
                    node.mother_name
                );

                if node_key == new_key {
                    insert_variation(&mut node.first_name_variations, first_name_var);
                    insert_variation(&mut node.last_name_variations, last_name_var);
                    insert_variation(&mut node.father_name_variations, father_name_var);
                    insert_variation(&mut node.grandfather_name_variations, grandfather_name_var);
                    insert_variation(&mut node.mother_last_name_variations, mother_last_name_var);
                    insert_variation(&mut node.mother_name_variations, mother_name_var);
                    return;
                }

                node_key > new_key
            }
            None => true,
        };

        if insert_here {
            let new_node = Box::new(IdentityNode {
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                father_name: father_name.to_string(),
                grandfather_name: grandfather_name.to_string(),
                mother_last_name: mother_last_name.to_string(),
                mother_name: mother_name.to_string(),
                dob,
                sex,
                place_of_birth: place_of_birth.to_string(),
                first_name_variations: Some(Box::new(VariationNode {
                    variation: first_name_var.to_string(),
                    next_variation: None,
                })),
                last_name_variations: Some(Box::new(VariationNode {
                    variation: last_name_var.to_string(),
                    next_variation: None,
                })),
                father_name_variations: Some(Box::new(VariationNode {
                    variation: father_name_var.to_string(),
                    next_variation: None,
                })),
                grandfather_name_variations: Some(Box::new(VariationNode {
                    variation: grandfather_name_var.to_string(),
                    next_variation: None,
                })),
                mother_last_name_variations: Some(Box::new(VariationNode {
                    variation: mother_last_name_var.to_string(),
                    next_variation: None,
                })),
                mother_name_variations: Some(Box::new(VariationNode {
                    variation: mother_name_var.to_string(),
                    next_variation: None,
                })),
                next_identity: cursor.take(),
            });

            *cursor = Some(new_node);
            return;
        }

        if let Some(node) = cursor {
            cursor = &mut node.next_identity;
        }
    }
}

// Rebuild the full identity dictionary from bulk records
pub fn rebuild_identity_dictionary(
    records: Vec<(
        String, String, String, String, String, String,
        Option<(u32, u32, u32)>,
        u8,
        String,
        String, String, String, String, String, String,
    )>,
) -> Option<Box<IdentityNode>> {
    let mut head = None;

    for (
        f, l, fa, g, ml, m, dob, sex, place,
        f_var, l_var, fa_var, g_var, ml_var, m_var,
    ) in records {
        insert_identity(
            &mut head,
            &f, &l, &fa, &g, &ml, &m,
            dob, sex, &place,
            &f_var, &l_var, &fa_var, &g_var, &ml_var, &m_var,
        );
    }

    head
}
impl IdentityNode {
    pub fn as_tuple(&self) -> (&str, &str, &str, &str, &str, &str) {
        (
            &self.first_name,
            &self.last_name,
            &self.father_name,
            &self.grandfather_name,
            &self.mother_last_name,
            &self.mother_name,
        )
    }
}
