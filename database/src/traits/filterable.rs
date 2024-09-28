#[macro_export]
macro_rules! impl_filterable {
    (
        $struct_name:ident,
        exact = [$($exact_field:ident),*],
        range = [$($range_field:ident),*],
        order_by = [ $( ($order_field:ident, $order_direction:ident) ),* $(,)? ]
    ) => {
        use sqlx::{postgres::PgArguments, Arguments};

        impl $struct_name {
            pub fn query(&self) -> String {
                let mut conditions = Vec::new();
                // let mut order = "ORDER BY ID ASC, created_at ASC".to_string();
                // if let Some(structure) = self.structure {
                //     match structure {
                //         "TransactionFilter"
                //     }
                // }

                $(
                    if self.$exact_field.is_some() {
                        conditions.push(format!("{} = ${}", stringify!($exact_field), conditions.len() + 1));
                    }
                )*

                $(
                    if let Some(ref range) = self.$range_field {
                        if range.start.is_some() {
                            conditions.push(format!("{} >= ${}", stringify!($range_field), conditions.len() + 1));
                        }

                        if range.end.is_some() {
                            conditions.push(format!("{} <= ${}", stringify!($range_field), conditions.len() + 1));
                        }
                    }
                )*

                let where_clause = if conditions.is_empty() {
                    String::new()
                } else {
                    "WHERE ".to_owned() + &conditions.join(" AND ")
                };

                let order_clause = {
                    let mut orders = Vec::new();
                    $(
                        orders.push(format!("{} {}", stringify!($order_field), stringify!($order_direction).to_uppercase()));
                    )*
                    if orders.is_empty() {
                        "ORDER BY ID ASC".to_string()
                    } else {
                        "ORDER BY ".to_owned() + &orders.join(", ")
                    }
                };

                let limit_clause = match (self.limit, self.offset) {
                    (Some(limit), _) => format!("LIMIT {}", limit),
                    (None, Some(_)) => format!("LIMIT {}", 25),
                    _ => String::new(),
                };

                let offset_clause = self.offset.map_or(String::new(), |o| format!("OFFSET {}", o));

                format!(
                    "{} {} {} {}",
                    where_clause,
                    order_clause,
                    offset_clause,
                    limit_clause,
                )
            }

            pub fn total(&self) -> String {
                let mut conditions = Vec::new();

                $(
                    if self.$exact_field.is_some() {
                            conditions.push(format!("{} = ${}", stringify!($exact_field), conditions.len() + 1));
                    }
                )*

                $(
                    if let Some(ref range) = self.$range_field {
                        if range.start.is_some() {
                            conditions.push(format!("{} >= ${}", stringify!($range_field), conditions.len() + 1));
                        }

                        if range.end.is_some() {
                            conditions.push(format!("{} <= ${}", stringify!($range_field), conditions.len() + 1));
                        }
                    }
                )*

                let where_clause = if conditions.is_empty() {
                    String::new()
                } else {
                    "WHERE ".to_owned() + &conditions.join(" AND ")
                };

                format!(
                    "{}",
                    where_clause,
                )
            }

            pub fn get_arguments(&self) -> PgArguments {
                let mut args = PgArguments::default();

                $(
                    if let Some(ref value) = self.$exact_field {
                        args.add(value);
                    }
                )*

                $(
                    if let Some(ref range) = self.$range_field {
                        if let Some(ref start) = range.start {
                            args.add(start);
                        }

                        if let Some(ref end) = range.end {
                            args.add(end);
                        }
                    }
                )*

                args
            }

            pub fn enforce_pagination(&mut self) {
                if self.limit.is_none() {
                    self.limit = Some(25);
                } else if self.limit.unwrap() < 5 {
                    self.limit = Some(5);
                } else if self.limit.unwrap() > 100 {
                    self.limit = Some(100);
                }

                if self.offset.is_none() {
                    self.offset = Some(0);
                }
            }
        }
    };
}
