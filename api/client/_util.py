def simplify_dicts(dict_list, desired_keys):
    return [{k: v for k, v in d.items() if k in desired_keys} for d in dict_list]
