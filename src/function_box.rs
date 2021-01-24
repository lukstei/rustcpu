use crate::connector::{Connector, ConnectorDirection};
use crate::game::PosF;

#[derive(Debug)]
pub struct FunctionBox {
    pub name: String,
    pub connectors: Vec<Connector>,
    output_start_idx: usize,
    pub outputs_len: usize,
    pub inputs_len: usize,

    pub position: PosF,
    pub generation: usize, // increased in every tick to avoid infinite recursion in circles
}

impl FunctionBox {
    pub fn get_input_connector(&self, name: &str) -> &Connector {
        self.inputs_iter().find(|x| { x.name == name }).unwrap()
    }
    pub fn get_output_connector(&self, name: &str) -> &Connector {
        self.outputs_iter().find(|x| { x.name == name }).unwrap()
    }

    pub fn inputs_iter(&self) -> impl Iterator<Item=&Connector> {
        self.connectors.iter().take(self.output_start_idx)
    }

    pub fn outputs_iter(&self) -> impl Iterator<Item=&Connector> {
        self.connectors.iter().skip(self.output_start_idx)
    }

    pub fn inputs_iter_mut(&mut self) -> impl Iterator<Item=&mut Connector> {
        self.connectors.iter_mut().take(self.output_start_idx)
    }

    pub fn outputs_iter_mut(&mut self) -> impl Iterator<Item=&mut Connector> {
        self.connectors.iter_mut().skip(self.output_start_idx)
    }

    pub(crate) fn new(name: &str, position: PosF, inputs: Vec<String>, outputs: Vec<String>) -> FunctionBox {
        let output_start_idx = inputs.len();

        FunctionBox {
            name: name.into(),
            output_start_idx,
            outputs_len: outputs.len(),
            inputs_len: inputs.len(),
            connectors:
            inputs.into_iter().enumerate()
                .chain(outputs.into_iter().enumerate()
                    .map(|(i, x)| { (i + output_start_idx, x) }))
                .map(|(i, n)| Connector::new(n, if i < output_start_idx { ConnectorDirection::Input } else { ConnectorDirection::Output }, i)).collect(),
            position,
            generation: 0,
        }
    }
}
