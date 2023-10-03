#include "basic_types.h"

#define PYPERF_STACK_WALKING_PROGRAM_IDX 0
// #define PYPERF_THREAD_STATE_PROGRAM_IDX 1

// Maximum Python stack frames: 16x5 = 80
#define PYTHON_STACK_FRAMES_PER_PROG 16
#define PYTHON_STACK_PROG_CNT 5
#define STACK_MAX_LEN (PYTHON_STACK_FRAMES_PER_PROG * PYTHON_STACK_PROG_CNT)
// rbperf
// #define MAX_STACKS_PER_PROGRAM 30
// #define BPF_PROGRAMS_COUNT 25
// #define MAX_STACK (MAX_STACKS_PER_PROGRAM * BPF_PROGRAMS_COUNT)

typedef struct {
    s64 ob_type;
} PyObject;

typedef struct {
    s64 data;
    s64 size;
} PyString;

typedef struct {
    s64 tp_name;
} PyTypeObject;

typedef struct {
    s64 next;
    s64 interp;
    s64 frame;
    s64 thread_id;
    s64 native_thread_id;

    s64 cframe;
} PyThreadState;

typedef struct {
    // since Python 3.11 this structure holds pointer to target FrameObject.
    s64 current_frame;
} PyCFrame;

typedef struct {
    s64 tstate_head;
} PyInterpreterState;

typedef struct {
    s64 interp_main;
} PyRuntimeState;

typedef struct {
    s64 f_back;
    s64 f_code;
    s64 f_lineno;
    s64 f_localsplus;
} PyFrameObject;

typedef struct {
    s64 co_filename;
    s64 co_name;
    s64 co_varnames;
    s64 co_firstlineno;
} PyCodeObject;

typedef struct {
    s64 ob_item;
} PyTupleObject;

// Offsets of structures across different Python versions:

// For the most part, these fields are named after their corresponding structures in Python.
// They are depicted as structures with 64-bit offset fields named after the requisite fields in the original structure.
// However, there are some deviations:
// 1. String - The offsets target the Python string object structure.
//     - Owing to the varying representation of strings across versions, which depends on encoding and interning,
//     the field names don't match those of a specific structure. Here, `data` is the offset pointing to the string's
//     first character, while `size` indicates the offset to the 32-bit integer that denotes the string's byte length
//     (not the character count).
// 2. PyRuntimeState.interp_main - This aligns with the offset of (_PyRuntimeState, interpreters.main).
// 3. PyThreadState.thread - In certain Python versions, this field is referred to as "thread_id".
typedef struct {
    u32 major_version;
    u32 minor_version;
    u32 patch_version;

    PyObject py_object;
    PyString py_string;
    PyTypeObject py_type_object;
    PyThreadState py_thread_state;
    PyCFrame py_cframe;
    PyInterpreterState py_interpreter_state;
    PyRuntimeState py_runtime_state;
    PyFrameObject py_frame_object;
    PyCodeObject py_code_object;
    PyTupleObject py_tuple_object;
} PythonVersionOffsets;

typedef struct {
    // u64 start_time;

    // u64 constant_buffer_addr;
    u64 interpreter_addr;
    u64 thread_state_addr;

    // TODO(kakkoyun): Try to obtain information in the runtime.
    // u64 tls_key_addr; // virtual address of autoTLSkey for pthreads TLS
    // u64 gil_locked_addr; // virtual address of gil_locked
    // u64 gil_last_holder_addr; // virtual address of gil_last_holder

    u32 py_version;
} ProcessInfo;

enum error_code {
    ERROR_NONE = 0,

    ERROR_MISSING_PYSTATE = 1,
    ERROR_THREAD_STATE_NULL = 2,
    ERROR_INTERPRETER_NULL = 3,

    ERROR_TOO_MANY_THREADS = 4,
    ERROR_THREAD_STATE_NOT_FOUND = 5,
    ERROR_EMPTY_STACK = 6,

    // ERROR_FRAME_CODE_IS_NULL = 7,
    ERROR_BAD_FSBASE = 8,
    ERROR_INVALID_PTHREADS_IMPL = 9,
    ERROR_THREAD_STATE_HEAD_NULL = 10,
    ERROR_BAD_THREAD_STATE = 11,
    ERROR_CALL_FAILED = 12,
    ERROR_TSTATE_CFRAME_IS_NULL = 13,
};

enum stack_status {
    STACK_COMPLETE = 0,
    STACK_TRUNCATED = 1,
    STACK_ERROR = 2,
};

// enum gil_state {
//     GIL_STATE_NO_INFO = 0,
//     GIL_STATE_ERROR = 1,
//     GIL_STATE_UNINITIALIZED = 2,
//     GIL_STATE_NOT_LOCKED = 3,
//     GIL_STATE_THIS_THREAD = 4,
//     GIL_STATE_GLOBAL_CURRENT_THREAD = 5,
//     GIL_STATE_OTHER_THREAD = 6,
//     GIL_STATE_NULL = 7,
// };

enum thread_state {
    THREAD_STATE_UNKNOWN = 0,
    THREAD_STATE_MATCH = 1,
    THREAD_STATE_MISMATCH = 2,
    THREAD_STATE_THIS_THREAD_NULL = 3,
    THREAD_STATE_GLOBAL_CURRENT_THREAD_NULL = 4,
    THREAD_STATE_BOTH_NULL = 5,
};

// enum pthread_id_match {
//     PTHREAD_ID_UNKNOWN = 0,
//     PTHREAD_ID_MATCH = 1,
//     PTHREAD_ID_MISMATCH = 2,
//     PTHREAD_ID_THREAD_STATE_NULL = 3,
//     PTHREAD_ID_NULL = 4,
//     PTHREAD_ID_ERROR = 5,
// };

#define COMM_LEN 16
#define CLASS_NAME_LEN 32
#define FUNCTION_NAME_LEN 64
#define FILE_NAME_LEN 128

typedef struct {
    char file[FILE_NAME_LEN];
    char class[CLASS_NAME_LEN];
    char func[FUNCTION_NAME_LEN];
    u32 line;
} Symbol;

// TODO(kakkoyun): Any useful fields could be moved to the Stack?
// struct event {
//     u32 stack_len;
//     s32 stack[STACK_MAX_LEN];

//     u64 user_ip;
//     u64 user_sp;

//     u32 user_stack_len;
//     uint8_t raw_user_stack[__USER_STACKS_PAGES__ * PAGE_SIZE];
// #define FRAME_CODE_IS_NULL 0x80000001
// };

typedef struct {
    s16 len;
    u32 frames[STACK_MAX_LEN];
} Stack;

typedef struct {
    pid_t pid;
    pid_t tid;
    int user_stack_id;
    int kernel_stack_id;
} stack_count_key_t;

typedef struct {
    u64 timestamp;
    u32 cpu;
    pid_t pid;
    pid_t tid;
    u8 comm[COMM_LEN];

    stack_count_key_t native_stack_count_key;

    enum stack_status stack_status;
    enum error_code error_code;

    // TODO(kakkoyun): Clean up
    // bool thread_current;
    // enum gil_state gil_state;
    // bool pthread_match;
    // enum pthread_id_match pthread_id_match;

    // TODO(kakkoyun): Shall we utilize this?
    // Stack related!
    // long long int size;
    // long long int expected_size;

    // int has_meta;
    // int metadata;
    // char dummy_safeguard;

    Stack stack;
} Sample;

typedef struct {
    ProcessInfo process_info;

    // void *interpreter;
    void *thread_state;

    // u64 current_thread_id;
    // int thread_state_prog_call_count;

    // TODO(kakkoyun): Unify naming.
    // TODO(kakkoyun): FrameData? FrameInfo?
    // u64 base_stack;  // TODO(kakkoyun): Where to find it? sp?
    // u64 cfp;
    // u64 sp;
    // u64 pc;
    // u64 bp;
    void *frame_ptr;
    int stack_walker_prog_call_count;

    Sample sample;
} State;
