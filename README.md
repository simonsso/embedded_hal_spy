# embedded-hal-spy
Implemenets rust embedded_hal for embedded_hal

embedded hal spy implemnets call backs for used traits

The intended use case is chaining over an existing embedded_hal implementation sniffing all the data and providing it back to a callback.  seful when preparing for a refacforing and want to collect actual data for unit test case.

