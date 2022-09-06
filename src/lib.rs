// Assumptions:
// - No gql imports using `require()`
// - Not removing duplicate declaration
// - No Directives are used
// - All declarations are defined as variable declaration
// - tag name is every where => gql - no need to test import statements for alias
// - No loc props

use apollo_parser::{
    ast::{
        Argument, Arguments, AstChildren, BooleanValue, DefaultValue, Definition, Directives,
        Document, EnumValue, Field, FloatValue, FragmentDefinition, FragmentSpread, InlineFragment,
        IntValue, ListType, ListValue, NamedType, NullValue, ObjectField, ObjectValue,
        OperationDefinition, OperationType, Selection, SelectionSet, StringValue, Type,
        TypeCondition, Value, Variable, VariableDefinition, VariableDefinitions,
    },
    Parser,
};

use regex::Regex;
use swc_common::Span;
use swc_core::{
    ast::*,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
    testing_transform::test,
    visit::{as_folder, FoldWith, VisitMut},
};

pub struct TransformVisitor;

fn parse_gql_string(body: String, span: Span) -> Option<Box<Expr>> {
    let parser = Parser::new(&body);
    let ast = parser.parse();
    assert_eq!(0, ast.errors().len());

    let doc = ast.document();

    create_document(doc, span)
}

impl VisitMut for TransformVisitor {
    fn visit_mut_var_decl(&mut self, node: &mut VarDecl) {
        let decls = &mut node.decls;
        for mut decl in decls {
            if let Some(initial) = &mut decl.init {
                if let Some(tag_tpl) = initial.as_mut_tagged_tpl() {
                    if let Some(tag) = tag_tpl.tag.as_mut_ident() {
                        if &tag.sym != "gql" {
                            return;
                        }

                        if tag_tpl.tpl.quasis.len() == 0 {
                            return;
                        }

                        let template = &mut tag_tpl.tpl;
                        let quasi = &mut template.quasis[0];
                        let data = &mut quasi.raw;
                        let gql_body = data.to_string();

                        // TODO: parse gql and insert it here
                        let gql_swc_ast = parse_gql_string(gql_body.clone(), tag_tpl.span);

                        decl.init = gql_swc_ast;
                    }
                }
            }
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}

fn create_key_value_prop(key: String, value: Expr) -> PropOrSpread {
    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Str(key.into()),
        value: Box::new(value),
    })))
}

fn create_document(document: Document, span: Span) -> Option<Box<Expr>> {
    let kind = create_key_value_prop("kind".into(), "Document".into());
    let definitions = create_key_value_prop(
        "definitions".into(),
        create_definitions(document.definitions(), span),
    );

    let document_object_lit = ObjectLit {
        span,
        props: vec![kind, definitions],
    };

    Some(Box::new(Expr::Object(document_object_lit)))
}

fn create_definitions(definitions: AstChildren<Definition>, span: Span) -> Expr {
    let mut all_definitions = vec![];
    for def in definitions {
        all_definitions.push(create_definition(def, span));
    }

    Expr::Array(ArrayLit {
        span,
        elems: all_definitions,
    })
}

fn create_definition(definition: Definition, span: Span) -> Option<ExprOrSpread> {
    let def_expr = match definition {
        Definition::FragmentDefinition(frag_def) => create_fragment_definition(frag_def, span),
        Definition::OperationDefinition(operation_def) => {
            create_operation_definition(operation_def, span)
        }
        Definition::DirectiveDefinition(_) => todo!(),
        Definition::SchemaDefinition(_) => todo!(),
        Definition::ScalarTypeDefinition(_) => todo!(),
        Definition::ObjectTypeDefinition(_) => todo!(),
        Definition::InterfaceTypeDefinition(_) => todo!(),
        Definition::UnionTypeDefinition(_) => todo!(),
        Definition::EnumTypeDefinition(_) => todo!(),
        Definition::InputObjectTypeDefinition(_) => todo!(),
        Definition::SchemaExtension(_) => todo!(),
        Definition::ScalarTypeExtension(_) => todo!(),
        Definition::ObjectTypeExtension(_) => todo!(),
        Definition::InterfaceTypeExtension(_) => todo!(),
        Definition::UnionTypeExtension(_) => todo!(),
        Definition::EnumTypeExtension(_) => todo!(),
        Definition::InputObjectTypeExtension(_) => todo!(),
    };

    Some(ExprOrSpread {
        spread: None,
        expr: def_expr,
    })
}

fn create_operation_definition(definition: OperationDefinition, span: Span) -> Box<Expr> {
    let kind = create_key_value_prop("kind".into(), "OperationDefinition".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(definition.name().unwrap().text().as_str().into(), span),
    );
    let variable_definitions = create_key_value_prop(
        "variableDefinitions".into(),
        create_variable_definitions(definition.variable_definitions(), span),
    );
    let directives = create_key_value_prop(
        "directives".into(),
        create_directives(definition.directives(), span),
    );
    let selection_set = create_key_value_prop(
        "selectionSet".into(),
        create_selection_set(definition.selection_set(), span),
    );
    let operation = create_key_value_prop(
        "operation".into(),
        get_operation_token(definition.operation_type()).into(),
    );

    let opr_def = ObjectLit {
        span,
        props: vec![
            kind,
            name,
            directives,
            selection_set,
            variable_definitions,
            operation,
        ],
    };

    Box::new(Expr::Object(opr_def))
}

fn create_fragment_definition(definition: FragmentDefinition, span: Span) -> Box<Expr> {
    let kind = create_key_value_prop("kind".into(), "FragmentDefinition".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(
            definition
                .fragment_name()
                .unwrap()
                .name()
                .unwrap()
                .text()
                .as_str()
                .into(),
            span,
        ),
    );
    let type_condition = create_key_value_prop(
        "typeCondition".into(),
        create_type_condition(definition.type_condition(), span),
    );
    let directives = create_key_value_prop(
        "directives".into(),
        create_directives(definition.directives(), span),
    );
    let selection_set = create_key_value_prop(
        "selectionSet".into(),
        create_selection_set(definition.selection_set(), span),
    );

    let frag_def = ObjectLit {
        span,
        props: vec![kind, name, type_condition, directives, selection_set],
    };

    Box::new(Expr::Object(frag_def))
}

fn create_variable_definitions(variable_defs: Option<VariableDefinitions>, span: Span) -> Expr {
    if variable_defs.is_none() {
        return Expr::Array(ArrayLit {
            span,
            elems: vec![],
        });
    }

    let mut all_variable_definitions = vec![];
    for variable_def in variable_defs.unwrap().variable_definitions() {
        all_variable_definitions.push(create_variable_definition(variable_def, span))
    }

    Expr::Array(ArrayLit {
        span,
        elems: all_variable_definitions,
    })
}

fn create_variable_definition(
    variable_def: VariableDefinition,
    span: Span,
) -> Option<ExprOrSpread> {
    let kind = create_key_value_prop("kind".into(), "VariableDefinition".into());
    let directives = create_key_value_prop(
        "directives".into(),
        create_directives(variable_def.directives(), span),
    );
    let variable = create_key_value_prop(
        "variable".into(),
        create_variable_value(variable_def.variable().unwrap(), span),
    );
    let default_value = create_key_value_prop(
        "defaultValue".into(),
        create_default_value(variable_def.default_value(), span),
    );
    let type_def = create_key_value_prop("type".into(), create_type(variable_def.ty(), span));

    let var_def = ObjectLit {
        span,
        props: vec![kind, directives, default_value, type_def, variable],
    };

    Some(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Object(var_def)),
    })
}

fn create_type(type_def: Option<Type>, span: Span) -> Expr {
    if type_def.is_none() {
        let type_object = ObjectLit {
            span,
            props: vec![],
        };

        return Expr::Object(type_object);
    }
    let unwrapped_type_def = type_def.unwrap();

    match unwrapped_type_def {
        Type::NamedType(named_type) => create_named_type(named_type, span),
        Type::ListType(list_type) => create_list_type(list_type, span),
        Type::NonNullType(_) => todo!(),
    }
}

fn create_named_type(named_type: NamedType, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "NamedType".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(named_type.name().unwrap().text().as_str().into(), span),
    );

    let type_object = ObjectLit {
        span,
        props: vec![kind, name],
    };

    Expr::Object(type_object)
}

fn create_list_type(list_type: ListType, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "ListType".into());
    let type_def = create_key_value_prop("type".into(), create_type(list_type.ty(), span));

    let type_object = ObjectLit {
        span,
        props: vec![kind, type_def],
    };

    Expr::Object(type_object)
}

fn create_name(name: String, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "Name".into());
    let value = create_key_value_prop("value".into(), name.into());
    let name = ObjectLit {
        span,
        props: vec![kind, value],
    };
    Expr::Object(name)
}

fn create_type_condition(type_condition: Option<TypeCondition>, span: Span) -> Expr {
    if type_condition.is_none() {
        let type_cond = ObjectLit {
            span,
            props: vec![],
        };
        return Expr::Object(type_cond);
    }

    let unwrapped_type_condition = type_condition.unwrap();

    let kind = create_key_value_prop("kind".into(), "NamedType".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(
            unwrapped_type_condition
                .named_type()
                .unwrap()
                .name()
                .unwrap()
                .text()
                .as_str()
                .into(),
            span,
        ),
    );

    let type_cond = ObjectLit {
        span,
        props: vec![kind, name],
    };
    Expr::Object(type_cond)
}

// TODO: add directives support if required
fn create_directives(_directives: Option<Directives>, span: Span) -> Expr {
    Expr::Array(ArrayLit {
        span,
        elems: vec![],
    })
}

fn create_selection_set(selection_set: Option<SelectionSet>, span: Span) -> Expr {
    if selection_set.is_none() {
        let sel_set = ObjectLit {
            span,
            props: vec![],
        };
        return Expr::Object(sel_set);
    }
    let unwrapped_selection_set = selection_set.unwrap();
    let kind = create_key_value_prop("kind".into(), "SelectionSet".into());
    let selections = create_key_value_prop(
        "selections".into(),
        create_selections(unwrapped_selection_set.selections(), span),
    );

    let sel_set = ObjectLit {
        span,
        props: vec![kind, selections],
    };
    Expr::Object(sel_set)
}

fn create_selections(selections: AstChildren<Selection>, span: Span) -> Expr {
    let mut all_selections = vec![];
    for selection in selections {
        all_selections.push(create_selection(selection, span));
    }

    Expr::Array(ArrayLit {
        span,
        elems: all_selections,
    })
}

fn create_selection(selection: Selection, span: Span) -> Option<ExprOrSpread> {
    match selection {
        Selection::Field(field) => create_field(field, span),
        Selection::FragmentSpread(frag_spread) => create_fragment_spread(frag_spread, span),
        Selection::InlineFragment(inline_frag) => create_inline_fragment(inline_frag, span),
    }
}

fn create_field(field: Field, span: Span) -> Option<ExprOrSpread> {
    let kind = create_key_value_prop("kind".into(), "Field".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(field.name().unwrap().text().as_str().into(), span),
    );
    let arguments = create_key_value_prop(
        "arguments".into(),
        create_arguments(field.arguments(), span),
    );
    let directives = create_key_value_prop(
        "directives".into(),
        create_directives(field.directives(), span),
    );
    let sel_set = create_key_value_prop(
        "selectionSet".into(),
        create_selection_set(field.selection_set(), span),
    );
    let sel = ObjectLit {
        span,
        props: vec![kind, name, arguments, directives, sel_set],
    };

    Some(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Object(sel)),
    })
}

fn create_fragment_spread(frag_spread: FragmentSpread, span: Span) -> Option<ExprOrSpread> {
    let kind = create_key_value_prop("kind".into(), "FragmentSpread".into());
    let name = create_key_value_prop(
        "name".into(),
        frag_spread
            .fragment_name()
            .unwrap()
            .name()
            .unwrap()
            .text()
            .as_str()
            .into(),
    );
    let directives = create_key_value_prop(
        "directives".into(),
        create_directives(frag_spread.directives(), span),
    );
    let fragment_spread = ObjectLit {
        span,
        props: vec![kind, name, directives],
    };

    Some(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Object(fragment_spread)),
    })
}

fn create_inline_fragment(inline_frag: InlineFragment, span: Span) -> Option<ExprOrSpread> {
    let kind = create_key_value_prop("kind".into(), "InlineFragment".into());
    let type_condition = create_key_value_prop(
        "typeCondition".into(),
        create_type_condition(inline_frag.type_condition(), span),
    );
    let directives = create_key_value_prop(
        "directives".into(),
        create_directives(inline_frag.directives(), span),
    );
    let sel_set = create_key_value_prop(
        "selectionSet".into(),
        create_selection_set(inline_frag.selection_set(), span),
    );

    let inline_frag_object = ObjectLit {
        span,
        props: vec![kind, type_condition, directives, sel_set],
    };

    Some(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Object(inline_frag_object)),
    })
}

fn create_arguments(arguments: Option<Arguments>, span: Span) -> Expr {
    if arguments.is_none() {
        let args = ArrayLit {
            span,
            elems: vec![],
        };
        return Expr::Array(args);
    }

    let unwrapped_arguments = arguments.unwrap().arguments();
    let mut all_arguments = vec![];
    for argument in unwrapped_arguments {
        all_arguments.push(create_argument(argument, span));
    }
    return Expr::Array(ArrayLit {
        span,
        elems: all_arguments,
    });
}

fn create_argument(argument: Argument, span: Span) -> Option<ExprOrSpread> {
    let kind = create_key_value_prop("kind".into(), "Argument".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(argument.name().unwrap().text().as_str().into(), span),
    );
    let value = create_key_value_prop("value".into(), create_value(argument.value(), span));
    let arg = ObjectLit {
        span,
        props: vec![kind, name, value],
    };
    Some(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Object(arg)),
    })
}

fn create_default_value(default_value: Option<DefaultValue>, span: Span) -> Expr {
    if default_value.is_none() {
        return Expr::Object(ObjectLit {
            span,
            props: vec![],
        });
    }

    let unwrapped_default_value = default_value.unwrap();
    create_value(unwrapped_default_value.value(), span)
}

fn create_value(value: Option<Value>, span: Span) -> Expr {
    assert!(value.is_some());
    let unwrapped_value = value.unwrap();
    match unwrapped_value {
        Value::Variable(var) => create_variable_value(var, span),
        Value::StringValue(str) => create_string_value(str, span),
        Value::FloatValue(float) => create_float_value(float, span),
        Value::IntValue(int) => create_int_value(int, span),
        Value::BooleanValue(bool) => create_boolean_value(bool, span),
        Value::NullValue(null) => create_null_value(null, span),
        Value::EnumValue(enum_val) => create_enum_value(enum_val, span),
        Value::ListValue(list) => create_list_value(list, span),
        Value::ObjectValue(object) => create_object_value(object, span),
    }
}

fn create_variable_value(var: Variable, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "Variable".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(var.name().unwrap().text().as_str().into(), span),
    );
    let variable = ObjectLit {
        span,
        props: vec![kind, name],
    };

    Expr::Object(variable)
}

fn create_string_value(str: StringValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "StringValue".into());

    let mut string_token = str.to_string();
    let re = Regex::new(r#""(?P<str>[^"]*)""#).unwrap();
    for cap in re.captures_iter(string_token.clone().as_str()) {
        string_token = cap[0].to_string();
    }
    let value = create_key_value_prop(
        "value".into(),
        string_token[1..string_token.len() - 1].into(),
    );

    let str_value = ObjectLit {
        span,
        props: vec![kind, value],
    };

    Expr::Object(str_value)
}

fn create_float_value(float: FloatValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "FloatValue".into());
    let value = create_key_value_prop("value".into(), float.float_token().unwrap().text().into());

    let float_val = ObjectLit {
        span,
        props: vec![kind, value],
    };

    Expr::Object(float_val)
}

fn create_int_value(int: IntValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "IntValue".into());
    let value = create_key_value_prop("value".into(), int.int_token().unwrap().text().into());

    let int_val = ObjectLit {
        span,
        props: vec![kind, value],
    };

    Expr::Object(int_val)
}

fn create_boolean_value(bool: BooleanValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "BooleanValue".into());
    let value = create_key_value_prop(
        "value".into(),
        (|| {
            if bool.true_token().is_some() {
                return Expr::Lit(Lit::Bool(true.into()));
            }
            return Expr::Lit(Lit::Bool(false.into()));
        })(),
    );

    let bool_val = ObjectLit {
        span,
        props: vec![kind, value],
    };

    Expr::Object(bool_val)
}

fn create_enum_value(enum_val: EnumValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "EnumValue".into());
    let value = create_key_value_prop("value".into(), enum_val.text().as_str().into());

    let enum_val_obj = ObjectLit {
        span,
        props: vec![kind, value],
    };

    Expr::Object(enum_val_obj)
}

fn create_null_value(_null: NullValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "NullValue".into());

    let null_val = ObjectLit {
        span,
        props: vec![kind],
    };

    Expr::Object(null_val)
}

fn create_list_value(list: ListValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "ListValue".into());
    let values = create_key_value_prop(
        "values".into(),
        create_list_value_values(list.values(), span),
    );

    let list_val = ObjectLit {
        span,
        props: vec![kind, values],
    };

    Expr::Object(list_val)
}

fn create_object_value(object: ObjectValue, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "ObjectValue".into());
    let fields = create_key_value_prop(
        "fields".into(),
        create_object_fields(object.object_fields(), span),
    );

    let object_val = ObjectLit {
        span,
        props: vec![kind, fields],
    };

    Expr::Object(object_val)
}

fn create_object_fields(object_fields: AstChildren<ObjectField>, span: Span) -> Expr {
    let mut all_fields = vec![];
    for field in object_fields.into_iter() {
        all_fields.push(Some(ExprOrSpread {
            spread: None,
            expr: Box::new(create_object_field(field, span)),
        }));
    }

    Expr::Array(ArrayLit {
        span,
        elems: all_fields,
    })
}

fn create_object_field(field: ObjectField, span: Span) -> Expr {
    let kind = create_key_value_prop("kind".into(), "ObjectField".into());
    let name = create_key_value_prop(
        "name".into(),
        create_name(field.name().unwrap().text().as_str().into(), span),
    );
    let value = create_key_value_prop("value".into(), create_value(field.value(), span));

    let object_field_value = ObjectLit {
        span,
        props: vec![kind, name, value],
    };

    Expr::Object(object_field_value)
}

fn create_list_value_values(values: AstChildren<Value>, span: Span) -> Expr {
    let mut all_values = vec![];
    for value in values.into_iter() {
        all_values.push(Some(ExprOrSpread {
            spread: None,
            expr: Box::new(create_value(Some(value), span)),
        }))
    }

    Expr::Array(ArrayLit {
        span,
        elems: all_values,
    })
}

fn get_operation_token(operation_type: Option<OperationType>) -> String {
    let opr_tokn = operation_type.unwrap();

    if opr_tokn.query_token().is_some() {
        return opr_tokn.query_token().unwrap().text().into();
    }

    if opr_tokn.mutation_token().is_some() {
        return opr_tokn.mutation_token().unwrap().text().into();
    }

    if opr_tokn.subscription_token().is_some() {
        return opr_tokn.subscription_token().unwrap().text().into();
    }

    "query".into()
}

test!(
    Default::default(),
    |_| as_folder(TransformVisitor),
    valid,
    // Input codes
    r#"const a = gql`
    query aaa ($a: I = "a", $b: b){
        user(id: $ss) {
          firstName
          lastName
        }
      }    `"#,
    // Output codes after transformed with plugin
    r#"const a = "apple""#
);
