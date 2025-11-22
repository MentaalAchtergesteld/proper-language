[FUNCTION <empty> (arity 0)]
 --- Constants ---
 0000 
 [FUNCTION <a> (arity 1)]
  --- Constants ---
  0000 
  [FUNCTION <c> (arity 1)]
   --- Constants ---
   --- Code ---
   0000 LoadLocal 0
   0001 Duplicate 1
   0002 StoreUpvalue 0
   0003 Pop 1
  [END FUNCTION <c>]
  --- Code ---
  0000 PushConst 0
  0001 LoadLocal 0
  0002 CreateClosure 1
  0003 StoreLocal 1
 [END FUNCTION <a>]
 0001 Integer 10
 0002 Integer 20
 --- Code ---
 0000 PushConst 0
 0001 CreateClosure 0
 0002 StoreLocal 0
 0003 LoadLocal 0
 0004 PushConst 1
 0005 Call 1
 0006 PushConst 2
 0007 Call 1
 0008 Pop 1
[END FUNCTION <empty>]
