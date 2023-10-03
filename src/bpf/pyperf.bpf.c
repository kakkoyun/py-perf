#include "pyperf.h"
#include "vmlinux.h"

#include <bpf/bpf_core_read.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ Constants and Configuration                                             ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//
const volatile bool verbose = false;

#define MAX_STACK_DEPTH 127
#define MAX_STACK_TRACES_ENTRIES 64000
#define MAX_STACK_COUNTS_ENTRIES 10240

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ Type Definitions                                                        ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//

#define EVENT_COMM_LEN 16
#define EVENT_MSG_LEN 256

// TODO(kakkoyun): Remove or use!
struct event {
    pid_t tid;
    pid_t pid;
    uid_t uid;
    u8 comm[EVENT_COMM_LEN];
    int kernel_stack_id;
    int user_stack_id;
    u8 msg[EVENT_MSG_LEN];
};

// Dummy instance for skeleton to generate definition.
struct event _event = {};

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ Macros                                                                  ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//
// TODO(kakkoyun): Remove cluttering abstractions.
// TODO(kakkoyun): Remove macros.
#define BPF_MAP(_name, _type, _key_type, _value_type, _max_entries) \
    struct {                                                        \
        __uint(type, _type);                                        \
        __uint(max_entries, _max_entries);                          \
        __type(key, _key_type);                                     \
        __type(value, _value_type);                                 \
    } _name SEC(".maps");

#define BPF_HASH(_name, _key_type, _value_type, _max_entries) \
    BPF_MAP(_name, BPF_MAP_TYPE_HASH, _key_type, _value_type, _max_entries);

typedef u64 stack_trace_type[MAX_STACK_DEPTH];
#define BPF_STACK_TRACE(_name, _max_entries) \
    BPF_MAP(_name, BPF_MAP_TYPE_STACK_TRACE, u32, stack_trace_type, _max_entries);

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║  BPF Maps                                                               ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//

struct {
    __uint(type, BPF_MAP_TYPE_PROG_ARRAY);
    __uint(max_entries, 3);
    __type(key, u32);
    __type(value, u32);
} programs SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
    __uint(key_size, sizeof(u32));
    __uint(value_size, sizeof(u32));
    __uint(max_entries, 8192);
} events SEC(".maps");

// struct {
//     __uint(type, BPF_MAP_TYPE_RINGBUF);
//     __uint(max_entries, 8192);
// } events SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 4096);
    __type(key, pid_t);
    __type(value, ProcessInfo);
} pid_to_process_info SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 10);
    __type(key, u32);
    __type(value, PythonVersionOffsets);
} version_specific_offsets SEC(".maps");

// struct {
//     __uint(type, BPF_MAP_TYPE_HASH);
//     __uint(max_entries, 1);
//     __type(key, int);
//     __type(value, event);
// } eventmap SEC(".maps");

// TODO(kakkoyun): Rename to sample!
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1);
    __type(key, int);
    __type(value, Stack);
} stackmap SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 1);
    __type(key, u32);
    __type(value, u64);
} symbol_index SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 64000);
    __type(key, Symbol);
    __type(value, int);
} symbols SEC(".maps");

BPF_STACK_TRACE(stack_traces, MAX_STACK_TRACES_ENTRIES);
BPF_HASH(stack_counts, stack_count_key_t, u64, MAX_STACK_COUNTS_ENTRIES);

struct {
    __uint(type, BPF_MAP_TYPE_PERCPU_ARRAY);
    __uint(max_entries, 1);
    __type(key, u32);
    __type(value, State);
} global_state SEC(".maps");

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ Generic Helpers                                                         ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//

#define GET_STATE()                                           \
    int zero = 0;                                             \
    State *state = bpf_map_lookup_elem(&global_state, &zero); \
    if (state == NULL) {                                      \
        return 0;                                             \
    }

#define GET_OFFSETS()                                                                                                \
    PythonVersionOffsets *offsets = bpf_map_lookup_elem(&version_specific_offsets, &state->process_info.py_version); \
    if (offsets == NULL) {                                                                                           \
        return 0;                                                                                                    \
    }

#define LOG(fmt, ...)                       \
    ({                                      \
        if (verbose) {                      \
            bpf_printk(fmt, ##__VA_ARGS__); \
        }                                   \
    })

static __always_inline void *bpf_map_lookup_or_try_init(void *map, const void *key, const void *init) {
    void *val;
    long err;

    val = bpf_map_lookup_elem(map, key);
    if (val) {
        return val;
    }

    err = bpf_map_update_elem(map, key, init, BPF_NOEXIST);
    if (err) {
        LOG("[error] bpf_map_lookup_or_try_init with ret: %d", err);
        return 0;
    }

    return bpf_map_lookup_elem(map, key);
}

static inline __attribute__((__always_inline__)) int submit_sample(struct bpf_perf_event_data *ctx, State *state) {
    LOG("[stop]");
    LOG("");
    // bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &event, sizeof(event));
    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &state->sample, sizeof(state->sample));
    // bpf_ringbuf_submit(stack, 0);
    // bpf_ringbuf_output(&events, &stack, sizeof(stack), 0);
    return 0;
}

// static inline __attribute__((__always_inline__)) int
// submit_event(struct bpf_perf_event_data *ctx, struct event *event) {
//     bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &event, sizeof(event));
//     // bpf_ringbuf_submit(event, 0);
//     // bpf_ringbuf_output(&events, &event, sizeof(event), 0);
//     return 0;
// }

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ Runtime Helpers                                                         ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//

// static inline __attribute__((__always_inline__)) void *
// get_interpreter(ProcessInfo *process_info) {

// }

// static inline __attribute__((__always_inline__)) void *
// get_thread_state(ProcessInfo *process_info) {
// }

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ BPF Programs                                                            ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//
SEC("perf_event")
int on_event(struct bpf_perf_event_data *ctx) {
    u64 pid_tgid = bpf_get_current_pid_tgid();
    pid_t pid = pid_tgid >> 32;
    pid_t tid = pid_tgid;

    if (pid == 0) {
        return 0;
    }

    ProcessInfo *process_info = bpf_map_lookup_elem(&pid_to_process_info, &pid);
    if (!process_info) {
        return 0;
    }

    LOG("[start]");
    LOG("[event] pid=%d tid=%d", pid, tid);

    if (process_info->thread_state_addr == 0) {
        LOG("[error] process_info.thread_state_addr was NULL");
        return 0;
    }

    // TODO(kakkoyun): Do or do not there is no try!
    // struct event *event = bpf_ringbuf_reserve(&events, sizeof(sample), 0);

    GET_STATE();
    // Reset state.
    state->process_info = (ProcessInfo){0};
    state->process_info = *process_info;
    // state->interpreter = 0;
    state->thread_state = 0;

    // state->base_stack = 0;
    // state->cfp = 0;
    state->frame_ptr = 0;
    state->stack_walker_prog_call_count = 0;

    state->sample = (Sample){0};
    state->sample.timestamp = bpf_ktime_get_ns();
    state->sample.tid = tid;
    state->sample.pid = pid;
    state->sample.cpu = bpf_get_smp_processor_id();
    bpf_get_current_comm(&state->sample.comm, sizeof(state->sample.comm));
    state->sample.native_stack_count_key = (stack_count_key_t){
        .pid = pid,
        .tid = tid,
        .kernel_stack_id = bpf_get_stackid(ctx, &stack_traces, 0),
        .user_stack_id = bpf_get_stackid(ctx, &stack_traces, BPF_F_USER_STACK),
    };
    state->sample.stack_status = STACK_ERROR;
    state->sample.error_code = ERROR_NONE;

    state->sample.stack = (Stack){0};
    state->sample.stack.len = 0;
    __builtin_memset((void *)state->sample.stack.frames, 0, sizeof(state->sample.stack.frames));

    u64 *scount = bpf_map_lookup_or_try_init(&stack_counts, &state->sample.native_stack_count_key, &zero);
    if (scount) {
        __sync_fetch_and_add(scount, 1);
    }

    // Fetch interpreter head.

    // LOG("process_info->interpreter_addr 0x%llx", process_info->interpreter_addr);
    // bpf_probe_read_user(&state->interpreter,
    //                     sizeof(state->interpreter),
    //                     (void *)(long)process_info->interpreter_addr);
    // LOG("interpreter 0x%llx", state->interpreter);

    // Fetch thread state.

    // GDB: ((PyThreadState *)_PyRuntime.gilstate.tstate_current)
    bpf_probe_read_user(&state->thread_state, sizeof(state->thread_state),
                        (void *)(long)process_info->thread_state_addr);
    LOG("process_info->thread_state_addr 0x%llx", process_info->thread_state_addr);
    LOG("thread_state 0x%llx", state->thread_state);

    // Read PyThreadState of this Thread from TLS.
    // void *thread_state = get_thread_state(tls_base, process_info);
    // if (!thread_state) {
    //     LOG("[error] thread_state was NULL");
    //     goto submit_event;
    // }

    // TODO(kakkoyun): THREAD STATE MATCH.
    // Check for matching between TLS PyThreadState and
    // the global _PyThreadState_Current.
    // event->thread_state_match =
    //     get_thread_state_match(thread_state, thread_state_current);

    // Read pthread ID of this Thread from TLS.

    // TODO(kakkoyun): Add function to get tls_base/fs_base.
    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    // This changes depending on arch and kernel version.
    // task->thread.fs, task->thread.tp_value, etc.
    long unsigned int tls_base = BPF_CORE_READ(task, thread.fsbase);
    LOG("tls_base 0x%llx", (void *)tls_base);

    GET_OFFSETS();

    // s64 thread_id;
    void *pthread_self, *pthread_created;
    bpf_probe_read_user(&pthread_created, sizeof(pthread_created),
                        state->thread_state + offsets->py_thread_state.thread_id);
    if (pthread_created == 0) {
        LOG("[error] pthread_created was NULL");
        goto submit_event;
    }
    LOG("pthread_created 0x%llx", pthread_created);
    // For __x86_64__, GLIBC
    // 0x10 = offsetof(struct pthread, header.self)
    // 0x10 = offsetof(tcbhead_t, self)
    bpf_probe_read_user(&pthread_self, sizeof(pthread_self), (void *)tls_base + 0x10);
    if (pthread_self == 0) {
        LOG("[error] pthread_self was NULL");
        goto submit_event;
    }
    LOG("pthread_self 0x%llx", pthread_self);

    // TODO(kakkoyun): PTHREAD ID MATCH.
    // Check for matching between pthread ID created current PyThreadState and
    // pthread of actual current pthread.
    // event->pthread_id_match =
    //     get_pthread_id_match(thread_state, tls_base, pid_data);

    // TODO(kakkoyun): GIL.
    // p (PyThreadState *)PyThread_tss_get(&_PyRuntime.gilstate.autoTSSkey)
    // // Read GIL state
    // event->gil_state =
    //     get_gil_state(thread_state, thread_state_current, pid_data);

    // TODO(kakkoyun): FRAME POINTER.
    if (state->thread_state == 0) {
        LOG("[error] thread_state was NULL");
        goto submit_event;
    }

    // TODO(kakkoyun): Better to check version.
    // Get pointer to top frame from PyThreadState.
    if (offsets->py_thread_state.frame > -1) {
        // TODO(kakkoyun): Maybe do this in user-space?!
        bpf_probe_read_user(&state->frame_ptr, sizeof(void *), state->thread_state + offsets->py_thread_state.frame);
    } else {
        void *cframe;
        bpf_probe_read_user(&cframe, sizeof(cframe), (void *)(state->thread_state + offsets->py_thread_state.cframe));
        if (cframe == 0) {
            LOG("[error] cframe was NULL");
            state->sample.error_code = ERROR_TSTATE_CFRAME_IS_NULL;
            goto submit_event;
        }
        LOG("cframe 0x%llx", cframe);

        bpf_probe_read_user(&state->frame_ptr, sizeof(state->frame_ptr),
                            (void *)(cframe + offsets->py_cframe.current_frame));
    }
    if (state->frame_ptr == 0) {
        LOG("[error] frame_ptr was NULL");
        state->sample.error_code = ERROR_EMPTY_STACK;
        goto submit_event;
    }

    LOG("frame_ptr 0x%llx", state->frame_ptr);
    bpf_tail_call(ctx, &programs, PYPERF_STACK_WALKING_PROGRAM_IDX);
    // bpf_tail_call(ctx, &programs, PYPERF_THREAD_STATE_PROGRAM_IDX);
    // This will never be executed.

submit_event:
    // TODO(kakkoyun): To tag or not to tag?!
    submit_sample(ctx, state);
    return 0;
}

static inline __attribute__((__always_inline__)) u64 get_symbol_id(Symbol *sym) {
    int *symbol_id_ptr = bpf_map_lookup_elem(&symbols, sym);
    if (symbol_id_ptr) {
        return *symbol_id_ptr;
    }

    u32 zero = 0;
    u64 *sym_idx = bpf_map_lookup_elem(&symbol_index, &zero);
    if (sym_idx == NULL) {
        // Appease the verifier, this will never fail.
        return 0;
    }

    u64 idx = __sync_fetch_and_add(sym_idx, 1);
    int err;
    err = bpf_map_update_elem(&symbols, sym, &idx, BPF_ANY);
    if (err) {
        LOG("[error] symbols failed with %d", err);
    }
    return idx;
}

// TODO(kakkoyun):
// ! Improve this function.
// * Add error handling.
// * Make sure we don't miss an edge case.
static inline __attribute__((__always_inline__)) void read_symbol(PythonVersionOffsets *offsets, void *cur_frame,
                                                                  void *code_ptr, Symbol *symbol) {
    // Figure out if we want to parse class name, basically checking the name of
    // the first argument.
    // If it's 'self', we get the type and it's name, if it's cls, we just get
    // the name. This is not perfect but there is no better way to figure this
    // out from the code object.
    // Everything we do in this function is best effort, we don't want to fail
    // the program if we can't read something.

    // GDB: ((PyTupleObject*)$frame->f_code->co_varnames)->ob_item[0]
    void *args_ptr;
    bpf_probe_read_user(&args_ptr, sizeof(void *), code_ptr + offsets->py_code_object.co_varnames);
    bpf_probe_read_user(&args_ptr, sizeof(void *), args_ptr + offsets->py_tuple_object.ob_item);
    bpf_probe_read_user_str(&symbol->func, sizeof(symbol->func), args_ptr + offsets->py_string.data);

    // Compare strings as ints to save instructions.
    char self_str[4] = {'s', 'e', 'l', 'f'};
    char cls_str[4] = {'c', 'l', 's', '\0'};
    bool first_self = *(s32 *)symbol->func == *(s32 *)self_str;
    bool first_cls = *(s32 *)symbol->func == *(s32 *)cls_str;

    // GDB: $frame->f_localsplus[0]->ob_type->tp_name.
    if (first_self || first_cls) {
        void *ptr;
        bpf_probe_read_user(&ptr, sizeof(void *), cur_frame + offsets->py_frame_object.f_localsplus);
        if (first_self) {
            // We are working with an instance, first we need to get type.
            bpf_probe_read_user(&ptr, sizeof(void *), ptr + offsets->py_object.ob_type);
        }
        bpf_probe_read_user(&ptr, sizeof(void *), ptr + offsets->py_type_object.tp_name);
        bpf_probe_read_user_str(&symbol->class, sizeof(symbol->class), ptr);
    }

    void *pystr_ptr;

    // GDB: $frame->f_code->co_filename
    bpf_probe_read_user(&pystr_ptr, sizeof(void *), code_ptr + offsets->py_code_object.co_filename);
    bpf_probe_read_user_str(&symbol->file, sizeof(symbol->file), pystr_ptr + offsets->py_string.data);

    // GDB: $frame->f_code->co_name
    bpf_probe_read_user(&pystr_ptr, sizeof(void *), code_ptr + offsets->py_code_object.co_name);
    bpf_probe_read_user_str(&symbol->func, sizeof(symbol->func), pystr_ptr + offsets->py_string.data);

    // GDB: $frame->f_code->co_firstlineno
    bpf_probe_read_user(&symbol->line, sizeof(symbol->line), code_ptr + offsets->py_code_object.co_firstlineno);
}

// TODO(kakkoyun): Decide.
// #define FAIL_COMPILATION_IF(condition)                \
//     typedef struct {                                  \
//         char _condition_check[1 - 2 * !!(condition)]; \
//     } STR_CONCAT(compile_time_condition_check, __COUNTER__);

// FAIL_COMPILATION_IF(sizeof(Symbol) == sizeof(struct bpf_perf_event_value))

static inline __attribute__((__always_inline__)) void reset_symbol(Symbol *sym) {
    __builtin_memset((void *)sym, 0, sizeof(Symbol));

    // We re-use the same Symbol instance across loop iterations, which means
    // we will have left-over data in the struct. Although this won't affect
    // correctness of the result because we have '\0' at end of the strings read,
    // it would affect effectiveness of the deduplication.
    // Helper bpf_perf_prog_read_value clears the buffer on error, so here we
    // (ab)use this behavior to clear the memory. It requires the size of Symbol
    // to be different from struct bpf_perf_event_value, which we check at
    // compilation time using the FAIL_COMPILATION_IF macro.
    // bpf_perf_prog_read_value(ctx, (struct bpf_perf_event_value *)sym, sizeof(Symbol));

    // sym->fn[0] = '\0';
    // sym->class[0] = '\0';
    // sym->file[0] = '\0';
}

SEC("perf_event")
int walk_python_stack(struct bpf_perf_event_data *ctx) {
    GET_STATE();
    GET_OFFSETS();

    LOG("[start] walk_python_stack");
    state->stack_walker_prog_call_count++;
    Sample *sample = &state->sample;

    // TODO(kakkoyun): Remove after testing.
    int frame_count = 0;
#pragma unroll
    for (int i = 0; i < PYTHON_STACK_FRAMES_PER_PROG; i++) {
        void *cur_frame = state->frame_ptr;
        if (!cur_frame) {
            break;
        }

        // Read the code pointer. PyFrameObject.f_code
        void *cur_code_ptr;
        bpf_probe_read_user(&cur_code_ptr, sizeof(cur_code_ptr), state->frame_ptr + offsets->py_frame_object.f_code);
        if (!cur_code_ptr) {
            LOG("[error] bpf_probe_read_user failed");
            break;
        }

        LOG("frame %d", frame_count);
        LOG("cur_frame_ptr 0x%llx", cur_frame);
        LOG("cur_code_ptr 0x%llx", cur_code_ptr);

        Symbol sym = (Symbol){0};
        reset_symbol(&sym);

        // Read symbol information from the code object if possible.
        read_symbol(offsets, cur_frame, cur_code_ptr, &sym);

        LOG("sym.file %s", sym.file);
        LOG("sym.class %s", sym.class);
        LOG("sym.fn %s", sym.func);
        LOG("sym.line %d", sym.line);

        u32 symbol_id = get_symbol_id(&sym);
        s64 cur_len = sample->stack.len;
        if (cur_len >= 0 && cur_len < STACK_MAX_LEN) {
            LOG("stack->frames[%d] = %d", cur_len, symbol_id);
            sample->stack.frames[cur_len] = symbol_id;
            sample->stack.len++;
        }
        frame_count++;

        bpf_probe_read_user(&state->frame_ptr, sizeof(state->frame_ptr), cur_frame + offsets->py_frame_object.f_back);
        if (!state->frame_ptr) {
            // There aren't any frames to read. We are done.
            goto complete;
        }
    }
    LOG("[iteration] frame_count %d", frame_count);

    LOG("state->stack_walker_prog_call_count %d", state->stack_walker_prog_call_count);
    if (state->stack_walker_prog_call_count < PYTHON_STACK_PROG_CNT) {
        LOG("[continue] walk_python_stack");
        bpf_tail_call(ctx, &programs, PYPERF_STACK_WALKING_PROGRAM_IDX);
        state->sample.error_code = ERROR_CALL_FAILED;
        goto submit;
    }

    LOG("[error] walk_python_stack TRUNCATED");
    LOG("[truncated] walk_python_stack, stack_len=%d", sample->stack.len);
    state->sample.error_code = ERROR_NONE;
    state->sample.stack_status = STACK_TRUNCATED;
    goto submit;

complete:
    LOG("[complete] walk_python_stack, stack_len=%d", sample->stack.len);
    state->sample.error_code = ERROR_NONE;
    state->sample.stack_status = STACK_COMPLETE;
submit:
    LOG("[stop] walk_python_stack");
    submit_sample(ctx, state);
    return 0;
}

//
//   ╔═════════════════════════════════════════════════════════════════════════╗
//   ║ Metadata                                                                ║
//   ╚═════════════════════════════════════════════════════════════════════════╝
//
#define KBUILD_MODNAME "py-perf"
volatile const char bpf_metadata_name[] SEC(".rodata") = "py-perf (https://github.com/kakkoyun/py-perf)";
unsigned int VERSION SEC("version") = 1;
char LICENSE[] SEC("license") = "GPL";
