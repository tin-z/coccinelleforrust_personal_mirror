// tokio fd36054ae4357686c33104619bfe0d04447f69ba
// illustrates typed metavariables
// 549e89e9cd2073ffa70f1bd12022c5543343be78 is similar

@@
RwLock<Slab<ScheduledIo>> lock;
@@

 lock.read()
- .unwrap()

@@
RwLock<Slab<ScheduledIo>> lock;
@@

 lock.write()
- .unwrap()
