extern crate num;
//use super::super::pins::pin::PinActions;
use crate::error::Error;
use eval;

#[derive(Debug)]
pub struct Timeset {
    pub name: String,
    pub period_as_string: Option<String>,
    pub default_period: Option<f64>,
}

impl Timeset {
    pub fn new(
        name: &str,
        period_as_string: Option<Box<dyn std::string::ToString>>,
        default_period: Option<f64>,
    ) -> Self {
        Timeset {
            name: String::from(name),
            period_as_string: match period_as_string {
                Some(p) => Some(p.to_string()),
                None => None,
            },
            default_period: default_period,
        }
    }

    pub fn eval_str(&self) -> String {
        let default = String::from("period");
        let p = self.period_as_string.as_ref().unwrap_or(&default);
        p.clone()
    }

    pub fn eval(&self, current_period: Option<f64>) -> Result<f64, Error> {
        let default = String::from("period");
        let p = self.period_as_string.as_ref().unwrap_or(&default);
        let err = &format!(
            "Could not evaluate Timeset {}'s expression: '{}'",
            self.name, p
        );
        if current_period.is_none() && self.default_period.is_none() {
            return Err(Error::new(&format!("No current timeset period set! Cannot evalate timeset '{}' without a current period as it does not have a default period!", self.name)));
        }
        match eval::Expr::new(p)
            .value(
                "period",
                current_period.unwrap_or(self.default_period.unwrap()),
            )
            .exec()
        {
            Ok(val) => match val.as_f64() {
                Some(v) => Ok(v),
                None => Err(Error::new(err)),
            },
            Err(_e) => Err(Error::new(err)),
        }
    }
}

impl Default for Timeset {
    fn default() -> Self {
        Self::new("dummy", Option::None, Option::None)
    }
}

// pub struct Event {
//   action: PinActions,
//   at: String,
// }

#[test]
fn test() {
    //let e = Event { action: PinActions::Drive, at: vec!() };
    // let t = Timeset::new("t1", Some(Box::new("1.0")), Option::None);
    // assert_eq!(t.eval(None).unwrap(), 1.0 as f64);

    let t = Timeset::new("t1", Some(Box::new("1.0 + 1")), Option::None);
    assert!(t.eval(None).is_err());

    let t = Timeset::new("t1", Some(Box::new("period")), Some(1.0 as f64));
    assert_eq!(t.eval(Some(1.0 as f64)).unwrap(), 1.0 as f64);

    let t = Timeset::new("t1", Some(Box::new("period + 0.25")), Some(1.0 as f64));
    assert_eq!(t.eval(Some(1.0 as f64)).unwrap(), 1.25 as f64);
}
