#include <cstdlib>
#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <filesystem>

using std::cout;
using std::endl;
using std::string;
using std::vector;

namespace fs = std::filesystem;
using fs::path;

const path BOOKMARKS_FILE = "/home/balder/.local/share/bookmarks/bookmarkss.txt";
const path ALIAS_FILE = "/home/balder/.local/share/bookmarks/aliases.txt";

void usage(){
    cout << "usage: bmark <command>" << endl << endl;
    cout << "Commands:" << endl;
    cout << "   add [<name>]    add a bookmark to the current working directory" << endl;
    cout << "   list            list all stored bookmarks" << endl;
    cout << "   edit            edit bookmarks in a text editor" << endl;
    cout << "   rm <name>       remove a bookmark with a given name" << endl;
    cout << "   update          update shell aliases file" << endl;
}

void add_bmark(string name = ""){
    path cwd = fs::current_path();

    if (name == "") name = cwd.stem();

    std::ofstream myfile (BOOKMARKS_FILE, std::ios_base::app); // Open in append mode
    if (myfile.is_open()){
        myfile << cwd.string() << " - " << name << endl;
        myfile.close();
    }
    else {
        cout << "ERROR: could not open file: " << BOOKMARKS_FILE << endl;
        exit(1);
    }
}

void list_bmark(){
    std::ifstream myfile (BOOKMARKS_FILE);
    string line;
    if (myfile.is_open()){
        while ( std::getline(myfile, line) ) {
            cout << line << endl;
        }
    }
    else {
        cout << "ERROR: could not open file: " << BOOKMARKS_FILE << endl;
        exit(1);
    }
}

void edit_bmark(){
    /* std::array<char> buffer[256]; */
    /* std::system(buffer); */
}

void rm_bmark(){

}

void update_bmark(){

}

int main(int argc, char **argv) {

    vector<string> args = {};

    for (int i = 0; i < argc; i ++){
        std::string arg = "";
        for (char* a = argv[i]; *a != '\0'; a++) {
            arg += *a;
        }
        args.push_back(arg);
    }

    if (args.size() == 1) {
        usage();
        exit(1);
    }

    if (args[1] == "add") {
        if (argc > 3) {
            cout << "ERROR: The `add` command only takes one argument." << endl;
            exit(1);
        }
        if (argc == 3) add_bmark(args[2]);
        else add_bmark();
    }
    else if (args[1] == "list") {
        list_bmark();
    }
    else if (args[1] == "edit") {
        edit_bmark();
    }
    else if (args[1] == "rm") {
        rm_bmark();
    }
    else if (args[1] == "update") {
        update_bmark();
    }
    else {
        usage();
        exit(1);
    }

    return 0;
}
