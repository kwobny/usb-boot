mod package_database;

use package_database::DatabaseFileDetails;

const DATABASE_FILE: DatabaseFileDetails = DatabaseFileDetails {
    xdg_dir_env_var: "XDG_CONFIG_HOME",
    home_dir_env_var: "HOME",
    default_xdg_dir: ".config",
    package_database_file: "custom-package-manager/",
};
