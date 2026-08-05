-- 种子数据：职能类别与部门

INSERT INTO categories (name, sort) VALUES
    ('宿舍', 1),
    ('食堂', 2),
    ('教学', 3),
    ('交通', 4),
    ('校园环境', 5),
    ('网络', 6),
    ('医疗健康', 7),
    ('学生服务', 8),
    ('其他', 99);

INSERT INTO departments (name, code, description) VALUES
    ('后勤实业发展中心', 'logistics', '食堂、宿舍、公共设施维护'),
    ('基建处', 'capital', '基建与维修'),
    ('信息化办公室', 'ict', '校园网络与信息系统'),
    ('保卫处', 'security', '安全保卫与交通秩序'),
    ('图书馆', 'library', '图书与研讨空间服务'),
    ('校医院', 'health', '医疗服务与健康咨询'),
    ('学生工作部', 'student-affairs', '学生事务与证明材料');
