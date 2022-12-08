var a = 15;
print a; // 15
{
	var b = 16;
	print b; // 16
	{
		var a = 17;
		print a; // 17
		a = 18;
		print a; // 18
		b = 19;
		print b; // 19
		var b = 20;
		print "inner b = " + b; // 20
	}
	print "outer b = " + b; // 19
}
print "global a = " + a; // 15