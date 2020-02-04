clean:
	rm -rf target/
	rm -rf esp_idf/target/
	rm -rf api/target/
	find . -name .DS_Store|xargs rm
