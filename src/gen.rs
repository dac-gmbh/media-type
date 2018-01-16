use quoted_string::quote_if_needed;
use percent_encoding::percent_encode;

use parse::ParamIndices;
use spec::Spec;
use error::ParserError;

const PARAM_SEP: &str = "; ";
const PARAM_KV_SEP: char = '=';
const TYPE_SEP: char = '/';
const PARAM_ENC_NAME_SUFFIC: char = '*';
const PARAM_ENC_VALUE_PREFIX: &str = "utf-8''";

//TODO replace key=value: (AsRef<str>, AsRef<str>) with T: KeyValue
// where trait KeyValue { fn key -> &str, fn value -> ??, fn lang_tag -> Option<&str> }
// with default impl for (&str, &str)
// not that fn value -> ?? has to work in a way that it can handle ->Value<- with to_content
// (we can't use repr, as we do not know if Value Spec if compatible with out Spec)

pub(crate) fn create_buffer_from<S>(
    type_: &str, subtype: &str
) -> Result<(String, usize, usize), ParserError>
    where S: Spec
{
    S::validate_token(type_)?;
    S::validate_token(subtype)?;

    let mut buffer = String::new();

    buffer.push_str(type_);
    let slash_idx = buffer.len();

    buffer.push(TYPE_SEP);
    buffer.push_str(subtype);
    let end_of_type = buffer.len();
    Ok((buffer, slash_idx, end_of_type))
}

pub(crate) fn push_params_to_buffer<S, I, IN, IV>(buffer: &mut String, params: I)
    -> Result<Vec<ParamIndices>, ParserError>
    where S: Spec,
          I: IntoIterator<Item=(IN,IV)>,
          IN: AsRef<str>,
          IV: AsRef<str>
{
    let mut param_indices = Vec::new();

    for (name, value) in params.into_iter() {
        let name = <IN as AsRef<str>>::as_ref(&name);
        let value = <IV as AsRef<str>>::as_ref(&value);
        S::validate_token(name)?;
        //TODO percent encode+split if value > threshold && it's MIME spec
        match quote_if_needed::<S, _>(value.as_ref(), &mut S::UnquotedValue::default()) {
            Ok(quoted_if_needed) => {
                //TODO if > threashold fall back to encodinf
                let value = quoted_if_needed.as_ref();
                let indices = buffer_push_param(buffer, name, value);
                param_indices.push(indices);
            },
            Err(_err) => {
                let indices = buffer_encode_and_push_param::<S>(buffer, name, value);
                param_indices.push(indices);
            }
        }
    }

    Ok(param_indices)
}

fn buffer_push_param(buffer: &mut String, name: &str, value: &str) -> ParamIndices {
    buffer.push_str(PARAM_SEP);
    let start = buffer.len();

    buffer.push_str(name);
    let eq_idx = buffer.len();

    buffer.push(PARAM_KV_SEP);
    buffer.push_str(value);
    let end = buffer.len();

    ParamIndices { start, eq_idx, end }
}

fn buffer_encode_and_push_param<S: Spec>(
    buffer: &mut String, name: &str, value: &str
) -> ParamIndices
{
    let encoded_value_parts =
        percent_encode(value.as_bytes(), S::PercentEncodeSet::default());

    buffer.push_str(PARAM_SEP);

    let start = buffer.len();
    buffer.push_str(name);
    buffer.push(PARAM_ENC_NAME_SUFFIC);

    let eq_idx = buffer.len();
    buffer.push(PARAM_KV_SEP);

    buffer.push_str(PARAM_ENC_VALUE_PREFIX);
    for value_part in encoded_value_parts {
        buffer.push_str(value_part);
    }
    let end = buffer.len();

    ParamIndices { start, eq_idx, end }
}