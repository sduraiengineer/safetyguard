Control loop:
* No memory allocation is allowed. The various IPC in linux involves memory allocation, where as shared memory doesn't. So shared memory is good for the use-case. Also, it is better to be lock-free. So Single producer and signle consumer queue with shared memory will be use as communication mechanism between apps and safetygurad.
* The application will update the shared memory and then send event via eventfd to indicate new data available in the shared memory. The safety guard will update the app specific details in the safetyguard.
* Also, there is timerfd in the control loop of safety guard. With this the state of all apps will be evaluvated. Such as heartbeats, memory-consuption details, or safeinstruments of the entire system. The timerfd is also responsible for update the watchdog of the system based on the monitored data. 
* 


