use quoted_string::quote_if_needed;
use percent_encoding::percent_encode;

use parse::ParamIndices;
use spec::Spec;
use error::Error;

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
) -> Result<(String, usize, usize), Error>
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

/// Push all parameters formatted to the output buffer
///
/// each parameter is preceded by "; " followed by <name> then "=",
/// then <value>. If the value needed to be quoted it will be quoted,
/// if the value needs to be encoded it's encoded (and "*" is added to the
/// parameter name.
///
/// # Error
///
/// an error is returned if a parameter name is not valid for the given
/// Spec `S`.
///
pub fn push_params_to_buffer<S, I, IN, IV>(buffer: &mut String, params: I)
    -> Result<Vec<ParamIndices>, Error>
    where S: Spec,
          I: IntoIterator<Item=(IN,IV)>,
          IN: AsRef<str>,
          IV: AsRef<str>
{
    let mut param_indices = Vec::new();

    for (name, value) in params.into_iter() {
        let name = <IN as AsRef<str>>::as_ref(&name);
        let value = <IV as AsRef<str>>::as_ref(&value);
        let indices = push_param_to_buffer::<S>(buffer, name, value)?;
        param_indices.push(indices);
    }

    Ok(param_indices)
}

/// Push one parameter formatted to the output buffer
///
/// the parameter is preceded by "; " followed by <name> then "=",
/// then <value>. If the value needed to be quoted it will be quoted,
/// if the value needs to be encoded it's encoded (and "*" is added to the
/// parameter name.
///
/// # Error
///
/// an error is returned if the parameter name is not valid for the given
/// Spec `S`.
///
pub fn push_param_to_buffer<S>(buffer: &mut String, name: &str, value: &str)
                                           -> Result<ParamIndices, Error>
    where S: Spec
{
    S::validate_token(name)?;
    //TODO percent encode+split if value > threshold && it's MIME spec
    //TODO important aboves TODO might change this TODO's interface
    Ok(match quote_if_needed::<S, _>(value, &mut S::UnquotedValue::default()) {
        Ok(quoted_if_needed) => {
            //TODO if > threashold fall back to encodinf
            let value = quoted_if_needed.as_ref();
            _buffer_push_param(buffer, name, value)
        },
        Err(_err) => {
            _buffer_encode_and_push_param::<S>(buffer, name, value)
        }
    })
}

fn _buffer_push_param(buffer: &mut String, name: &str, value: &str) -> ParamIndices {
    buffer.push_str(PARAM_SEP);
    let start = buffer.len();

    buffer.push_str(name);
    let eq_idx = buffer.len();

    buffer.push(PARAM_KV_SEP);
    buffer.push_str(value);
    let end = buffer.len();

    ParamIndices { start, eq_idx, end }
}

fn _buffer_encode_and_push_param<S: Spec>(
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