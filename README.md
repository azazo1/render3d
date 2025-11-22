# render3d

使用 rust 手撕一个简单的光线追踪 3D 渲染场景.

参考学习代码(Andrew Kensler):

```cpp
#include <stdlib.h>
#include <stdio.h>
#include <math.h>
typedef int i;typedef float f;struct v{f x,y,z;v operator+(v r){return v(x+r.x,y+r.y,z+r.z);}v operator*(f r){return v(x*r,y*r,z*r);}f operator%(v r){return x*r.x+y*r.y+z*r.z;}v(){}v operator^(v r){return v(y*r.z-z*r.y,z*r.x-x*r.z,x*r.y-y*r.x);}v(f a,f b,f c){x=a;y=b;z=c;}v operator!(){return*this*(1/sqrt(*this%*this));}};i G[]={247570,280596,280600,249748,18578,18577,231184,16,16};f R(){return(f)rand()/RAND_MAX;}i T(v o,v d,f&t,v&n){t=1e9;i m=0;f p=-o.z/d.z;if(.01<p)t=p,n=v(0,0,1),m=1;for(i k=19;k--;)for(i j=9;j--;)if(G[j]&1<<k){v p=o+v(-k,0,-j-4);f b=p%d,c=p%p-1,q=b*b-c;if(q>0){f s=-b-sqrt(q);if(s<t&&s>.01)t=s,n=!(p+d*t),m=2;}}return m;}v S(v o,v d){f t;v n;i m=T(o,d,t,n);if(!m)return v(.7,.6,1)*pow(1-d.z,4);v h=o+d*t,l=!(v(9+R(),9+R(),16)+h*-1),r=d+n*(n%d*-2);f b=l%n;if(b<0||T(h,l,t,n))b=0;f p=pow(l%r*(b>0),99);if(m&1){h=h*.2;return((i)(ceil(h.x)+ceil(h.y))&1?v(3,1,1):v(3,3,3))*(b*.2+.1);}return v(p,p,p)+S(h,r)*.5;}i main(){printf("P6 512 512 255 ");v g=!v(-6,-16,0),a=!(v(0,0,1)^g)*.002,b=!(g^a)*.002,c=(a+b)*-256+g;for(i y=512;y--;)for(i x=512;x--;){v p(13,13,13);for(i r=64;r--;){v t=a*(R()-.5)*99+b*(R()-.5)*99;p=S(v(17,16,8)+t,!(t*-1+(a*(R()+x)+b*(y+R())+c)*16))*3.5+p;}printf("%c%c%c",(i)p.x,(i)p.y,(i)p.z);}}
```

gemini 3 pro 的解释:

```cpp
#include <stdlib.h> // rand()
#include <stdio.h>  // printf()
#include <math.h>   // sqrt(), pow()

// 定义简单的类型别名，原代码为了省空间
// Vector 结构体：代表点(x,y,z) 或 向量(x,y,z)
struct Vec {
    float x, y, z;

    // 构造函数
    Vec(float a = 0, float b = 0, float c = 0) { x = a; y = b; z = c; }

    // 向量加法: v1 + v2
    Vec operator+(Vec r) { return Vec(x + r.x, y + r.y, z + r.z); }

    // 向量乘标量: v * f
    Vec operator*(float r) { return Vec(x * r, y * r, z * r); }

    // 【重要技巧】原代码用 % 重载了点积 (Dot Product)
    // 数学含义：a·b = ax*bx + ay*by + az*bz
    float dot(Vec r) { return x * r.x + y * r.y + z * r.z; }

    // 【重要技巧】原代码用 ^ 重载了叉积 (Cross Product)
    // 得到一个垂直于这两个向量的新向量
    Vec cross(Vec r) { return Vec(y * r.z - z * r.y, z * r.x - x * r.z, x * r.y - y * r.x); }

    // 【重要技巧】原代码用 ! 重载了归一化 (Normalize)
    // 把向量长度变为 1，方向不变
    Vec normalize() { return *this * (1 / sqrt(this->dot(*this))); }
};

// 场景数据：这其实是一个位图数组
// 每一位的 0 或 1 决定了空间中该位置是否放置一个球体
// 这些数字拼起来其实是 "P ixar" 或者类似的字样
int data[] = {247570, 280596, 280600, 249748, 18578, 18577, 231184, 16, 16};

// 生成 0.0 到 1.0 之间的随机数
float random_float() { return (float)rand() / RAND_MAX; }

// 【核心函数 1】光线求交测试 (Trace)
// origin: 光线起点, dir: 光线方向
// t: 输出参数，记录击中物体的距离
// normal: 输出参数，记录击中点的法线
// 返回值: 0=没击中, 1=击中地板, 2=击中球体
int intersect(Vec origin, Vec dir, float& t, Vec& normal) {
    t = 1e9; // 初始化距离为无穷大
    int material = 0; // 默认材质 0 (天空/无)

    // --- 检测与地板的相交 (平面方程 z = 0) ---
    // p 是光线射向地板所需的距离 (-origin.z / dir.z)
    float p = -origin.z / dir.z;
    if (p > 0.01) { // 如果距离为正（在地板上方往下看）
        t = p;
        normal = Vec(0, 0, 1); // 地板法线永远向上 (0,0,1)
        material = 1; // 材质 1：地板
    }

    // --- 检测与球体的相交 ---
    // 这是一个双层循环，遍历空间网格
    // 19 是 x 轴范围，9 是 z 轴范围 (y 轴由位图控制)
    for (int k = 19; k--;) {
        for (int j = 9; j--;) {
            // 检查位图 data[j] 的第 k 位是否为 1
            if (data[j] & (1 << k)) {
                // 如果是 1，说明这里有一个球
                // 球心位置 p (注意原代码这里复用了 p 这个变量名)
                Vec center = Vec(-k, 0, -j - 4); // 定义球的位置
                Vec sphere_to_ray = origin + center;

                // 解一元二次方程求光线与球的交点: at^2 + bt + c = 0
                // 因为光线方向已归一化，a=1，简化了计算
                float b = sphere_to_ray.dot(dir);
                float c = sphere_to_ray.dot(sphere_to_ray) - 1; // 1 是半径的平方
                float discriminant = b * b - c; // 判别式 delta

                if (discriminant > 0) { // 有交点
                    float s = -b - sqrt(discriminant); // 近处的交点距离
                    if (s < t && s > 0.01) { // 如果这个球比之前碰到的物体更近
                        t = s;
                        normal = (sphere_to_ray + dir * t).normalize(); // 计算球表面法线
                        material = 2; // 材质 2：球体
                    }
                }
            }
        }
    }
    return material;
}

// 【核心函数 2】着色器 / 采样 (Sample)
// 计算光线打到的颜色
Vec radiance(Vec origin, Vec dir) {
    float t;
    Vec normal;

    // 1. 调用求交函数，看看光线打到了什么
    int m = intersect(origin, dir, t, normal);

    // 情况 A: 没打中任何东西 (m=0)，返回天空颜色
    if (!m) {
        // 基于光线方向的 z 分量做一个简单的渐变色
        return Vec(0.7, 0.6, 1) * pow(1 - dir.z, 4);
    }

    // 情况 B: 打中了物体 (m=1 地板, m=2 球体)

    // 计算击中点坐标: hit_point = origin + dir * t
    Vec hit_point = origin + dir * t;

    // --- 阴影与光照计算 ---

    // 定义一个面光源的大致方向和随机扰动（实现软阴影）
    Vec light_pos = Vec(9 + random_float(), 9 + random_float(), 16);
    Vec light_dir = (light_pos + hit_point * -1).normalize(); // 指向光源的向量

    // 计算兰伯特(Lambertian) 漫反射强度: N dot L
    float lambert = normal.dot(light_dir);
    if (lambert < 0) lambert = 0; // 背光面为全黑

    // 发射一条阴影光线(Shadow Ray)去检查光源是否被遮挡
    float shadow_t;
    Vec shadow_n;
    // 注意：这里从击中点向光源发射光线
    if (intersect(hit_point, light_dir, shadow_t, shadow_n)) {
        lambert = 0; // 如果中途撞到了东西，说明在阴影里，亮度置为 0
    }

    // 计算高光 (Phong Specular)
    // r 是光线的反射方向
    Vec r = dir + normal * (normal.dot(dir) * -2);
    // 这里的 99 是高光指数，指数越大光斑越小越亮
    float specular = pow(light_dir.dot(r) * (lambert > 0), 99);

    // --- 材质颜色处理 ---

    if (m & 1) { // m=1, 地板
        hit_point = hit_point * 0.2; // 缩放纹理坐标
        // 棋盘格纹理：根据 x 和 y 的整数部分奇偶性决定颜色
        int is_white = (int)(ceil(hit_point.x) + ceil(hit_point.y)) & 1;
        // 返回：(红色或白色) * 漫反射亮度
        return (is_white ? Vec(3, 1, 1) : Vec(3, 3, 3)) * (lambert * 0.2 + 0.1);
    }
    else { // m=2, 球体 (镜面反射)
        // 递归调用 radiance 函数，计算反射光的颜色，这就实现了镜面反射
        // 0.5 是反射率，表示球体反射 50% 的光
        return Vec(specular, specular, specular) + radiance(hit_point, r) * 0.5;
    }
}

int main() {
    // PPM 图片格式头: P6 宽 高 最大颜色值
    printf("P6 512 512 255 ");

    // --- 摄像机坐标系构建 ---
    // 这里的数字是为了找一个好的拍摄角度
    Vec g = Vec(-6, -16, 0).normalize(); // 视线方向 (Gaze)
    // 叉积算出右向量 (a) 和 上向量 (b)
    Vec a = Vec(0, 0, 1).cross(g).normalize() * 0.002;
    Vec b = g.cross(a).normalize() * 0.002;
    // 图像平面的左上角位置
    Vec c = (a + b) * -256 + g;

    // 遍历像素 (512x512)
    for (int y = 512; y--;) {
        for (int x = 512; x--;) {
            Vec pixel_color(13, 13, 13); // 初始颜色微偏亮，避免全黑

            // 蒙特卡洛采样 (Monte Carlo Sampling)
            // 每个像素发射 64 条光线取平均值，消除锯齿并产生景深/软阴影效果
            for (int r = 64; r--;) {
                // 在镜头光圈上随机取一点 (实现景深 DOF)
                Vec t = a * (random_float() - 0.5) * 99 + b * (random_float() - 0.5) * 99;

                // 计算通过像素 (x, y) 的光线方向
                Vec ray_dir = (t * -1 + (a * (random_float() + x) + b * (y + random_float()) + c) * 16).normalize();

                // 累加颜色，3.5 是曝光补偿
                pixel_color = radiance(Vec(17, 16, 8) + t, ray_dir) * 3.5 + pixel_color;
            }

            // 输出颜色 (简单的 Gamma 矫正和类型转换)
            printf("%c%c%c", (int)pixel_color.x, (int)pixel_color.y, (int)pixel_color.z);
        }
    }
    return 0;
}
```
