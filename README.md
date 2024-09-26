# Bug using `edge-executor` or `async-executor` on ESP32-S3 with PSRAM enabled

This example code should be flashed onto an ESP32-S3 with integrated PSRAM to reproduce the issues.

Steps:

1) Compile and flash the code to the ESP with `cargo run --release`
2) After the ESP booted, connect to the now open `esp-animation` WIFI network with the password `hello-world`
3) Open multiple windows of a Chromium-based browser on the address `http://192.168.71.1/`
    - Firefox does not seem to be able to display the same MJPEG stream in multiple tabs or windows
    - Tests were done with six small browser windows on a second monitor
    - In each window, a repeating animation of a jumping ball should be displayed
4) After a random amount of time, the following might happen:
    - One or more of the streams might freeze, the first one often within a minute or so
    - The ESP crashes with one of different kinds of exceptions (see the collection below)

These issues happen when using `edge-executor` or `async-executor`. The executor can be selected by uncommenting one of the `http_xxx()` lines at the bottom of the `main()` function.

There are multiple ways to stop these issues from happening:

* Disable RSRAM by commenting out or removing all SPIRAM related options in the `sdkconfig.defaults`
* Using a different executor like `futures-executor::LocalPool` or `emwait::LocalExecutor`

    **Note:** To use the `emwait` Executor, the `emwait` dependency in the `Cargo.toml`, as well as the `http_emwait()` function in the `main.rs` must be uncommented. This requires access to the private repository!

Executor         |   PSRAM enabled  | PSRAM disabled
-----------------|------------------|----------------
edge-executor    | freezes, crashes |     works
async-executor   |     freezes      |     works
emwait           |      works       |     works
futures-executor |      works       |     works

## Collection of Stack Traces

```
Guru Meditation Error: Core  1 panic'ed (LoadProhibited). Exception was unhandled.

Core  1 register dump:
PC      : 0x42032915  PS      : 0x00060b30  A0      : 0x82032b43  A1      : 0x3fcc05d0  
0x42032915 - tcp_free_acked_segments$isra$0
    at ??:??
A2      : 0x3fcc527c  A3      : 0x3fc9f04c  A4      : 0x000005a0  A5      : 0x3fcc1dc1  
0x3fc9f04c - ackno
    at ??:??
A6      : 0x3fc9f048  A7      : 0x3fcc1dc1  A8      : 0x3fcc1dc1  A9      : 0x00000000  
0x3fc9f048 - recv_acked
    at ??:??
A10     : 0x183fcc71  A11     : 0x000005a0  A12     : 0x00000000  A13     : 0x3fcc3170  
A14     : 0x00060023  A15     : 0x00000003  SAR     : 0x0000001a  EXCCAUSE: 0x0000001c  
EXCVADDR: 0x183fcc76  LBEG    : 0x4209ab54  LEND    : 0x4209ab5b  LCOUNT  : 0x00000000  
0x4209ab54 - lwip_standard_chksum
    at ??:??
0x4209ab5b - lwip_standard_chksum
    at ??:??


Backtrace: 0x42032912:0x3fcc05d0 0x42032b40:0x3fcc0600 0x42034283:0x3fcc0630 0x42039382:0x3fcc0670 0x4203cfe2:0x3fcc06a0 0x4202f2d1:0x3fcc06c0
0x42032912 - tcp_free_acked_segments$isra$0
    at ??:??
0x42032b40 - tcp_receive
    at ??:??
0x42034283 - tcp_input
    at ??:??
0x42039382 - ip4_input
    at ??:??
0x4203cfe2 - ethernet_input
    at ??:??
0x4202f2d1 - tcpip_thread
    at ??:??
```

```
Guru Meditation Error: Core  0 panic'ed (LoadProhibited). Exception was unhandled.

Core  0 register dump:
PC      : 0x42032915  PS      : 0x00060f30  A0      : 0x82032b43  A1      : 0x3fcc05d0  
0x42032915 - tcp_free_acked_segments$isra$0
    at ??:??
A2      : 0x3fcc47a8  A3      : 0x3fc9f04c  A4      : 0x000005a0  A5      : 0x3fcc4989  
0x3fc9f04c - ackno
    at ??:??
A6      : 0x3fc9f048  A7      : 0x3fcc4989  A8      : 0x3fcc4989  A9      : 0x00000000  
0x3fc9f048 - recv_acked
    at ??:??
A10     : 0x183fccdf  A11     : 0x000005a0  A12     : 0x000005a0  A13     : 0x000000cf  
A14     : 0x00000001  A15     : 0x00000047  SAR     : 0x0000001a  EXCCAUSE: 0x0000001c  
EXCVADDR: 0x183fcce4  LBEG    : 0x4209ab54  LEND    : 0x4209ab5b  LCOUNT  : 0x00000000  
0x4209ab54 - lwip_standard_chksum
    at ??:??
0x4209ab5b - lwip_standard_chksum
    at ??:??


Backtrace: 0x42032912:0x3fcc05d0 0x42032b40:0x3fcc0600 0x42034283:0x3fcc0630 0x42039382:0x3fcc0670 0x4203cfe2:0x3fcc06a0 0x4202f2d1:0x3fcc06c0
0x42032912 - tcp_free_acked_segments$isra$0
    at ??:??
0x42032b40 - tcp_receive
    at ??:??
0x42034283 - tcp_input
    at ??:??
0x42039382 - ip4_input
    at ??:??
0x4203cfe2 - ethernet_input
    at ??:??
0x4202f2d1 - tcpip_thread
    at ??:??
```

```
Guru Meditation Error: Core  0 panic'ed (StoreProhibited). Exception was unhandled.

Core  0 register dump:
PC      : 0x40382be5  PS      : 0x00060333  A0      : 0x80382652  A1      : 0x3fcabb70  
0x40382be5 - tlsf_malloc
    at ??:??
A2      : 0x00000000  A3      : 0x3fca209c  A4      : 0x4203cf4c  A5      : 0x00060323  
0x4203cf4c - ethernet_input
    at ??:??
A6      : 0x3fc9eec0  A7      : 0x3fca2068  A8      : 0x3fca2044  A9      : 0x3fca2044  
0x3fc9eec0 - socket_ipv4_multicast_memberships
    at ??:??
A10     : 0x3fccc4c5  A11     : 0x00000010  A12     : 0x44000000  A13     : 0x803fca20  
A14     : 0x00000000  A15     : 0x00000006  SAR     : 0x0000001c  EXCCAUSE: 0x0000001d  
EXCVADDR: 0x803fca2c  LBEG    : 0x40056f5c  LEND    : 0x40056f72  LCOUNT  : 0xffffffff  


Backtrace: 0x40382be2:0x3fcabb70 0x4038264f:0x3fcabb90 0x403763b7:0x3fcabbb0 0x40376402:0x3fcabbe0 0x403848fd:0x3fcabc00 0x4202f54c:0x3fcabc20 0x4202f5f4:0x3fcabc40 0x4202f633:0x3fcabc60 0x4202f33b:0x3fcabc80 0x4202f37d:0x3fcabca0 0x42040d7e:0x3fcabcc0 0x4209abfd:0x3fcabce0 0x420a0bb9:0x3fcabd00 0x42065e6d:0x3fcabd20 0x42067ca6:0x3fcabd40 0x42065ff5:0x3fcabda0 0x40386b7d:0x3fcabdc0 0x40385677:0x3fcabde0
0x40382be2 - tlsf_malloc
    at ??:??
0x4038264f - multi_heap_malloc_impl
    at ??:??
0x403763b7 - heap_caps_malloc_base
    at ??:??
0x40376402 - heap_caps_malloc_default
    at ??:??
0x403848fd - pvalloc
    at ??:??
0x4202f54c - mem_malloc
    at ??:??
0x4202f5f4 - do_memp_malloc_pool$isra$0
    at ??:??
0x4202f633 - memp_malloc
    at ??:??
0x4202f33b - tcpip_inpkt
    at ??:??
0x4202f37d - tcpip_input
    at ??:??
0x42040d7e - wlanif_input
    at ??:??
0x4209abfd - esp_netif_receive
    at ??:??
0x420a0bb9 - wifi_ap_receive
    at ??:??
0x42065e6d - hostap_deliver_data
    at ??:??
0x42067ca6 - hostap_input
    at ??:??
0x42065ff5 - ap_rx_cb
    at ??:??
0x40386b7d - ppRxPkt
    at ??:??
0x40385677 - ppTask
    at ??:??
```

```
Guru Meditation Error: Core  0 panic'ed (LoadStoreError). Exception was unhandled.

Core  0 register dump:
PC      : 0x4038001b  PS      : 0x00060b33  A0      : 0x82010595  A1      : 0x3fcc3350  
0x4038001b - xTaskGenericNotify
    at ??:??
A2      : 0x4203598c  A3      : 0x00000000  A4      : 0x00000001  A5      : 0x00000001  
0x4203598c - tcpip_tcp_timer
    at ??:??
A6      : 0x00000000  A7      : 0x3fca80e4  A8      : 0x8038000a  A9      : 0x42035a8c  
0x42035a8c - sys_untimeout
    at ??:??
A10     : 0x00000001  A11     : 0xffffffff  A12     : 0x00060b20  A13     : 0x00060b23  
A14     : 0x3fca8130  A15     : 0x0000cdcd  SAR     : 0x0000001f  EXCCAUSE: 0x00000003  
EXCVADDR: 0x42035adc  LBEG    : 0x4202aafc  LEND    : 0x4202abc2  LCOUNT  : 0x00000000  
0x42035adc - sys_timeouts_sleeptime
    at ??:??
0x4202aafc - set_global_fd_sets
    at ??:??
0x4202abc2 - set_global_fd_sets
    at ??:??


Backtrace: 0x40380018:0x3fcc3350 0x42010592:0x3fcc3370 0x4200e7e9:0x3fcc33a0 0x42002f97:0x3fcc33c0 0x4200a695:0x3fcc3400 0x4200ee86:0x3fcc3430 0x4200f957:0x3fcc3550 0x4202161b:0x3fcc3580 0x42021b08:0x3fcc35a0
0x40380018 - xTaskGenericNotify
    at ??:??
0x42010592 - <esp_idf_hal::task::notification::Notifier as alloc::task::Wake>::wake
    at ??:??
0x4200e7e9 - alloc::task::raw_waker::wake
    at ??:??
0x42002f97 - <F as async_task::runnable::Schedule<M>>::schedule
    at ??:??
0x4200a695 - async_task::raw::RawTask<F,T,S,M>::wake
    at ??:??
0x4200ee86 - std::sys_common::backtrace::__rust_begin_short_backtrace
    at ??:??
0x4200f957 - core::ops::function::FnOnce::call_once{{vtable.shim}}
    at ??:??
0x4202161b - std::sys::pal::unix::thread::Thread::new::thread_start
    at ??:??
0x42021b08 - pthread_task_func
    at ??:??
```