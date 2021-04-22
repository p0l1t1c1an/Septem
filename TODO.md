## ToDo

## Applications should act as server
  - It spawns other threads running handlers other functionality

## Threads act as clients
  - They request info from a channel made with application 
  - Application may then request/push something from other client or already contain info
  - Then, it returns the request or sends the push info to relevant clients
  - This is how the event handler sends data on the new pid for the tracker to know a new event has occurred

## Multiple Config Structs
  - Create a struct from each configurable thread 
    - So for event_handler, recorder and so on
  - Then, we can create defaults for each of them 

## Create system to start objects after parsing config
  - First parse config for each threads config options (or load defaults)
  - Then, we can create each object async and use try_join to wait for all them to finish
  - Then, we run "start" on each object spawning a new thread for them

