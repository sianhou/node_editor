use eframe::egui::WidgetType::DragValue;
use eframe::egui::{Ui, Widget};
use eframe::{
    egui::{self, DragValue},
    epi,
};

use egui_node_graph::*;

pub struct MyNodeData {
    template: MyNodeTemplate,
}

#[derive(PartialEq, Eq)]
pub enum MyDataType {
    Scalar,
    Vec2,
}

#[derive(Copy, Clone, Debug)]
pub enum MyValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
}

impl MyValueType {
    pub fn try_to_vec2(self) -> anyhow::Result<egui::Vec2> {
        if let MyValueType::Vec2 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec2", self);
        }
    }

    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let MyValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }
}

#[derive(Clone, Copy)]
pub enum MyNodeTemplate {
    MakeVector,
    MakeScalar,
    AddScalar,
    SubtractScalar,
    VectorTimesScalar,
    AddVector,
    SubtractVector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
}

#[derive(Default)]
pub struct MyGraphState {
    pub active_node: Option<NodeId>,
}

impl DataTypeTrait for MyDataType {
    fn data_type_color(&self) -> egui::Color32 {
        match self {
            MyDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            MyDataType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
        }
    }

    fn name(&self) -> &str {
        match self {
            MyDataType::Scalar => "scalar",
            MyDataType::Vec2 => "2d vector",
        }
    }
}

impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    fn node_finder_label(&self) -> &str {
        match self {
            MyNodeTemplate::MakeVector => "New vector",
            MyNodeTemplate::MakeScalar => "New scalar",
            MyNodeTemplate::AddScalar => "Scalar add",
            MyNodeTemplate::SubtractScalar => "Scalar subtract",
            MyNodeTemplate::AddVector => "Vector add",
            MyNodeTemplate::SubtractVector => "Vector subtract",
            MyNodeTemplate::VectorTimesScalar => "Vector times scalar",
        }
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        MyNodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId,
    ) {
        macro_rules! input {
            (scalar $name:expr) => {
                graph.add_input_param(
                    node_id,
                    $name.to_string(),
                    MyDataType::Scalar,
                    MyValueType::Scalar { value: 0.0 },
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
            };
            (vector $name:expr) => {
                graph.add_input_param(
                    node_id,
                    $name.to_string(),
                    MyDataType::Vec2,
                    MyValueType::Vec2 {
                        value: egui::vec2(0.0, 0.0),
                    },
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
            };
        }

        macro_rules! output {
            (scalar $name:expr) => {
                graph.add_output_param(node_id, $name.to_string(), MyDataType::Scalar);
            };
            (vector $name:expr) => {
                graph.add_output_param(node_id, $name.to_string(), MyDataType::Vec2);
            };
        }

        match self {
            MyNodeTemplate::AddScalar => {
                graph.add_input_param(
                    node_id,
                    "A".into(),
                    MyDataType::Scalar,
                    MyValueType::Scalar { value: 0.0 },
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                input!(scalar "B");
                output!(scalar "out");
            }
            MyNodeTemplate::SubtractScalar => {
                input!(scalar "A");
                input!(scalar "B");
                output!(scalar "out");
            }
            MyNodeTemplate::VectorTimesScalar => {
                input!(scalar "scalar");
                input!(vector "vector");
                output!(vector "out");
            }
            MyNodeTemplate::AddVector => {
                input!(vector "v1");
                input!(vector "v2");
                output!(vector "out");
            }
            MyNodeTemplate::SubtractVector => {
                input!(vector "v1");
                input!(vector "v2");
                output!(vector "out");
            }
            MyNodeTemplate::MakeVector => {
                input!(scalar "x");
                input!(scalar "y");
                output!(vector "out");
            }
            MyNodeTemplate::MakeScalar => {
                input!(scalar "value");
                output!(scalar "out");
            }
        }
    }
}

pub struct AllMyNodeTemplates;

impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = MyNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            MyNodeTemplate::MakeScalar,
            MyNodeTemplate::MakeVector,
            MyNodeTemplate::AddScalar,
            MyNodeTemplate::SubtractScalar,
            MyNodeTemplate::AddVector,
            MyNodeTemplate::SubtractVector,
            MyNodeTemplate::VectorTimesScalar,
        ]
    }
}

impl WidgetValueTrait for MyValueType {
    fn value_widget(&mut self, param_name: &str, ui: &mut Ui) {
        match self {
            MyValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value.y));
                });
            }
            MyValueType::Scalar { value } => ui.horizontal(|ui| {
                ui.label(param_name);
                ui.add(DragValue::new(value));
            }),
        }
    }
}

impl UserResponseTrait for MyResponse {}

impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    fn bottom_ui(
        &self,
        ui: &mut Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &Self::UserState,
    ) -> Vec<NodeResponse<Self::Response>>
    where
        Self::Response: UserResponseTrait,
    {
        let mut responses = vec![];
        let is_active = user_state
            .active_node
            .map(|id| id == node_id)
            .unwrap_or(false);

        if !is_active {
            if ui.button("???? Set active").clicked(){
                responses.push(NodeResponse::User(MyResponse::SetActiveNode(node_id)));
            } else {
                egui::Button::new(egui::RichText::new("???? Active").color(egui::Color32::BLACK))
                    .fill(egui::Color32::GOLD);
                if ui.add(button).clicked() {
                    responses.push(NodeResponse::User(MyResponse::ClearActiveNode));
                }
            }

        }
        responses
    }
}

type MyGraph = Graph<MyNodeData, MyDataType,MyValueType>;
type MyEditorState = GraphEditorState<MyNodeData,MyDataType,MyValueType,MyNodeTemplate,MyGraphState>;

