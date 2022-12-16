print "before";
if (!true) {
	print "never";
} else {
	print "sanity restored";
}
if (true) {
	print "good";
} else {
	print "never";
}
print "after";

// Expected output: before, sanity restored, good, after