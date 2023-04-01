#include <cstdlib>
#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <filesystem>

using std::string;

void usage();
void add_bmark(string name = "");
void list_bmark();
void edit_bmark();
void open_bmark();
void rm_bmark();
void update_bmark();
int main(int argc, char **argv) ;
