use memoffset::offset_of;
use std::fs::File;
use std::io::Write;
use std::mem::size_of;
use std::path::Path;

use py_perf::bindings::PythonVersionOffsets;

static OUT_DIR: &str = "src/python_versions";

fn write_to_file(filename: &str, contents: PythonVersionOffsets) {
    let yaml = serde_yaml::to_string(&contents).unwrap();

    File::create(Path::new(OUT_DIR).join(filename))
        .unwrap()
        .write_all(yaml.as_bytes())
        .unwrap();
}

fn dump_python_structs_2_7_15() {
    let python_2_7_15_offsets = PythonVersionOffsets {
        major_version: 2,
        minor_version: 7,
        patch_version: 15,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v2_7_15::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: offset_of!(py_spy::python_bindings::v2_7_15::PyStringObject, ob_sval) as i64,
            size: offset_of!(py_spy::python_bindings::v2_7_15::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v2_7_15::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v2_7_15::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v2_7_15::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v2_7_15::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v2_7_15::PyThreadState, thread_id)
                as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v2_7_15::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v2_7_15::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v2_7_15::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v2_7_15::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(
                py_spy::python_bindings::v2_7_15::PyFrameObject,
                f_localsplus
            ) as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v2_7_15::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v2_7_15::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v2_7_15::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v2_7_15::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v2_7_15::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_2_7_15.yaml", python_2_7_15_offsets)
}

fn dump_python_structs_3_3_7() {
    let python_3_3_7_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 3,
        patch_version: 7,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_3_7::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: size_of::<py_spy::python_bindings::v3_3_7::PyASCIIObject>() as i64,
            size: offset_of!(py_spy::python_bindings::v3_3_7::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_3_7::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_3_7::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_3_7::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_3_7::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_3_7::PyThreadState, thread_id) as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_3_7::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_3_7::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_3_7::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_3_7::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(py_spy::python_bindings::v3_3_7::PyFrameObject, f_localsplus)
                as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_3_7::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_3_7::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_3_7::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_3_7::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_3_7::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_3_7.yaml", python_3_3_7_offsets)
}

fn dump_python_structs_3_5_5() {
    let python_3_5_5_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 5,
        patch_version: 5,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_5_5::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: size_of::<py_spy::python_bindings::v3_5_5::PyASCIIObject>() as i64,
            size: offset_of!(py_spy::python_bindings::v3_5_5::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_5_5::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_5_5::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_5_5::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_5_5::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_5_5::PyThreadState, thread_id) as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_5_5::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_5_5::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_5_5::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_5_5::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(py_spy::python_bindings::v3_5_5::PyFrameObject, f_localsplus)
                as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_5_5::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_5_5::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_5_5::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_5_5::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_5_5::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_5_5.yaml", python_3_5_5_offsets)
}

fn dump_python_structs_3_6_6() {
    let python_3_6_6_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 6,
        patch_version: 6,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_6_6::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: size_of::<py_spy::python_bindings::v3_6_6::PyASCIIObject>() as i64,
            size: offset_of!(py_spy::python_bindings::v3_6_6::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_6_6::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_6_6::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_6_6::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_6_6::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_6_6::PyThreadState, thread_id) as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_6_6::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_6_6::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_6_6::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_6_6::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(py_spy::python_bindings::v3_6_6::PyFrameObject, f_localsplus)
                as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_6_6::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_6_6::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_6_6::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_6_6::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_6_6::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_6_6.yaml", python_3_6_6_offsets)
}

fn dump_python_structs_3_7_0() {
    let python_3_7_0_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 7,
        patch_version: 0,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_7_0::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: size_of::<py_spy::python_bindings::v3_7_0::PyASCIIObject>() as i64,
            size: offset_of!(py_spy::python_bindings::v3_7_0::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_7_0::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_7_0::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_7_0::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_7_0::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_7_0::PyThreadState, thread_id) as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_7_0::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_7_0::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_7_0::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_7_0::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(py_spy::python_bindings::v3_7_0::PyFrameObject, f_localsplus)
                as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_7_0::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_7_0::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_7_0::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_7_0::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_7_0::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_7_0.yaml", python_3_7_0_offsets)
}

fn dump_python_structs_3_8_0() {
    let python_3_8_0_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 8,
        patch_version: 0,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_8_0::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: size_of::<py_spy::python_bindings::v3_8_0::PyASCIIObject>() as i64,
            size: offset_of!(py_spy::python_bindings::v3_8_0::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_8_0::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_8_0::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_8_0::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_8_0::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_8_0::PyThreadState, thread_id) as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_8_0::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_8_0::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_8_0::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_8_0::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(py_spy::python_bindings::v3_8_0::PyFrameObject, f_localsplus)
                as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_8_0::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_8_0::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_8_0::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_8_0::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_8_0::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_8_0.yaml", python_3_8_0_offsets)
}

fn dump_python_structs_3_9_5() {
    let python_3_9_5_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 9,
        patch_version: 5,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_9_5::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            data: size_of::<py_spy::python_bindings::v3_9_5::PyASCIIObject>() as i64,
            size: offset_of!(py_spy::python_bindings::v3_9_5::PyVarObject, ob_size) as i64,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_9_5::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_9_5::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_9_5::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_9_5::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_9_5::PyThreadState, thread_id) as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_9_5::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_9_5::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_9_5::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_9_5::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(py_spy::python_bindings::v3_9_5::PyFrameObject, f_localsplus)
                as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_9_5::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_9_5::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_9_5::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_9_5::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_9_5::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_9_5.yaml", python_3_9_5_offsets)
}

fn dump_python_structs_3_10_0() {
    let python_3_10_0_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 10,
        patch_version: 0,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_10_0::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            // see https://github.com/python/cpython/blob/3.10/Include/cpython/unicodeobject.h#L82-L84
            data: size_of::<py_spy::python_bindings::v3_10_0::PyASCIIObject>() as i64,
            size: -1,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_10_0::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_10_0::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_10_0::PyThreadState, next) as i64,
            frame: offset_of!(py_spy::python_bindings::v3_10_0::PyThreadState, frame) as i64,
            thread_id: offset_of!(py_spy::python_bindings::v3_10_0::PyThreadState, thread_id)
                as i64,
            native_thread_id: -1,
            cframe: -1,
        },
        py_cframe: py_perf::bindings::PyCFrame::default(),
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_10_0::PyInterpreterState,
                tstate_head
            ) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState { interp_main: -1 },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(py_spy::python_bindings::v3_10_0::PyFrameObject, f_back) as i64,
            f_code: offset_of!(py_spy::python_bindings::v3_10_0::PyFrameObject, f_code) as i64,
            f_lineno: offset_of!(py_spy::python_bindings::v3_10_0::PyFrameObject, f_lineno) as i64,
            f_localsplus: offset_of!(
                py_spy::python_bindings::v3_10_0::PyFrameObject,
                f_localsplus
            ) as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_10_0::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_10_0::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(py_spy::python_bindings::v3_10_0::PyCodeObject, co_varnames)
                as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_10_0::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_10_0::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_10_0.yaml", python_3_10_0_offsets)
}

fn dump_python_structs_3_11_0() {
    let python_3_11_0_offsets = PythonVersionOffsets {
        major_version: 3,
        minor_version: 11,
        patch_version: 0,
        py_object: py_perf::bindings::PyObject {
            ob_type: offset_of!(py_spy::python_bindings::v3_11_0::PyObject, ob_type) as i64,
        },
        py_string: py_perf::bindings::PyString {
            // see https://github.com/python/cpython/blob/3.11/Include/cpython/unicodeobject.h#L69-L71
            data: size_of::<py_spy::python_bindings::v3_11_0::PyASCIIObject>() as i64,
            size: -1,
        },
        py_type_object: py_perf::bindings::PyTypeObject {
            tp_name: offset_of!(py_spy::python_bindings::v3_11_0::PyTypeObject, tp_name) as i64,
        },
        py_thread_state: py_perf::bindings::PyThreadState {
            interp: offset_of!(py_spy::python_bindings::v3_11_0::PyThreadState, interp) as i64,
            next: offset_of!(py_spy::python_bindings::v3_11_0::PyThreadState, next) as i64,
            frame: -1,
            thread_id: offset_of!(py_spy::python_bindings::v3_11_0::PyThreadState, thread_id)
                as i64,
            native_thread_id: offset_of!(
                py_spy::python_bindings::v3_11_0::PyThreadState,
                native_thread_id
            ) as i64,
            // pointer to intermediate structure, PyCFrame.
            cframe: offset_of!(py_spy::python_bindings::v3_11_0::PyThreadState, cframe) as i64,
        },
        py_cframe: py_perf::bindings::PyCFrame {
            current_frame: offset_of!(py_spy::python_bindings::v3_11_0::_PyCFrame, current_frame)
                as i64,
        },
        py_interpreter_state: py_perf::bindings::PyInterpreterState {
            tstate_head: offset_of!(
                py_spy::python_bindings::v3_11_0::PyInterpreterState,
                threads
            ) as i64
                + offset_of!(py_spy::python_bindings::v3_11_0::_is_pythreads, head) as i64,
        },
        py_runtime_state: py_perf::bindings::PyRuntimeState {
            interp_main: offset_of!(
                py_spy::python_bindings::v3_11_0::pyruntimestate,
                interpreters
            ) as i64
                + offset_of!(
                    py_spy::python_bindings::v3_11_0::pyruntimestate_pyinterpreters,
                    main
                ) as i64,
        },
        py_frame_object: py_perf::bindings::PyFrameObject {
            f_back: offset_of!(
                py_spy::python_bindings::v3_11_0::_PyInterpreterFrame,
                previous
            ) as i64,
            f_code: offset_of!(
                py_spy::python_bindings::v3_11_0::_PyInterpreterFrame,
                f_code
            ) as i64,
            f_lineno: -1,
            f_localsplus: offset_of!(
                py_spy::python_bindings::v3_11_0::_PyInterpreterFrame,
                localsplus
            ) as i64,
        },
        py_code_object: py_perf::bindings::PyCodeObject {
            co_filename: offset_of!(py_spy::python_bindings::v3_11_0::PyCodeObject, co_filename)
                as i64,
            co_name: offset_of!(py_spy::python_bindings::v3_11_0::PyCodeObject, co_name) as i64,
            co_varnames: offset_of!(
                py_spy::python_bindings::v3_11_0::PyCodeObject,
                co_localsplusnames
            ) as i64,
            co_firstlineno: offset_of!(
                py_spy::python_bindings::v3_11_0::PyCodeObject,
                co_firstlineno
            ) as i64,
        },
        py_tuple_object: py_perf::bindings::PyTupleObject {
            ob_item: offset_of!(py_spy::python_bindings::v3_11_0::PyTupleObject, ob_item) as i64,
        },
    };

    write_to_file("python_3_11_0.yaml", python_3_11_0_offsets)
}

fn main() {
    dump_python_structs_2_7_15();

    dump_python_structs_3_3_7();
    dump_python_structs_3_5_5();

    dump_python_structs_3_6_6();
    dump_python_structs_3_7_0();
    dump_python_structs_3_8_0();
    dump_python_structs_3_9_5();
    dump_python_structs_3_10_0();
    dump_python_structs_3_11_0();
}
