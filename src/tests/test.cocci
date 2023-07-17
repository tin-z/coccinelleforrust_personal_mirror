@ rule1 @
expression x, y;
@@

-Lrc<ty::CrateVariancesMap<'tcx>>
+&'tcx ty::CrateVariancesMap<'tcx>