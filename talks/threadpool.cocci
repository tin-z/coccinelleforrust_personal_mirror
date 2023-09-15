// tokio 1f91a890b4ff6f3707970d1c15350469f72c1a68
// there are only two occurrences of this change, but this is a cleanup of
// uses of old APIs, so maybe it was more useful before

@@
identifier sched_tx, _scheduler;
symbol threadpool;
@@

-        let (sched_tx, _scheduler) = Pool::new();
+        let threadpool = ThreadPool::new();
         <... when != _scheduler
              when != sched_tx
-        sched_tx.execute
+        threadpool
           (...)
-          .ok().unwrap()
         ...>
